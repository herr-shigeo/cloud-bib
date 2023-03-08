use crate::error::*;
use crate::item::atoi;
use crate::item::BarcodeSetting;
use crate::item::RentalSetting;
use crate::item::SystemSetting;
use crate::item::{search_item, search_items, update_item};
use crate::item::{Book, BorrowedBook, User};
use crate::views::cache::*;
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::views::session::*;
use crate::views::transaction::*;
use crate::views::utils::get_nowtime;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use log::{debug, error, info};
use mongodb::Database;
use serde::Deserialize;
use shared_mongodb::database::{abort_transaction, commit_transaction, start_transaction};
use shared_mongodb::{database, ClientHolder};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct ProcessWorkForm {
    pub user_id: String,
    pub borrowed_book_id: String,
    pub returned_book_id: String,
}

pub async fn process(
    session: Session,
    form: web::Json<ProcessWorkForm>,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<Mutex<HashMap<String, SystemSetting>>>,
    cache_map: web::Data<Mutex<HashMap<String, Cache>>>,
    transaction_map: web::Data<Mutex<HashMap<String, Transaction>>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    let dbname = check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let setting_map = setting_map.lock().unwrap();
    let system_setting = setting_map.get(&dbname);
    if system_setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let system_setting = system_setting.unwrap().clone();
    drop(setting_map);

    // Verify the digits of the barcodes
    let barcode_setting = BarcodeSetting::default();
    let mut barcode_setting = match search_items(&db, &barcode_setting).await {
        Ok(barcode_setting) => barcode_setting,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };
    if barcode_setting.len() != 1 {
        return Err(BibErrorResponse::DataDuplicated(0));
    }
    let barcode_setting = barcode_setting.pop().unwrap();

    check_digits_of_user_barcodes(&barcode_setting, &form.user_id)?;
    check_digits_of_book_barcodes(&barcode_setting, &form.borrowed_book_id)?;
    check_digits_of_book_barcodes(&barcode_setting, &form.returned_book_id)?;

    let mut user = User::default();
    if form.user_id == "" && form.borrowed_book_id == "" && form.returned_book_id != "" {
        let (book_title, book_id) = unborrow_book(
            &db,
            &dbname,
            &cache_map,
            &transaction_map,
            &mut user,
            &form.returned_book_id,
            &system_setting.time_zone,
        )
        .await?;
        let mut reply = Reply::default();
        reply.returned_book_title = book_title;
        reply.returned_book_id = book_id;
        reply.user = user;
        return Ok(HttpResponse::Ok().json(reply));
    }

    user.id = atoi(&form.user_id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    let mut user = match search_item(&db, &user).await {
        Ok(user) => user,
        Err(_) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::UserNotFound(user.id));
        }
    };

    let rental_setting = RentalSetting::default();
    let mut rental_setting = match search_items(&db, &rental_setting).await {
        Ok(rental_setting) => rental_setting,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };
    if rental_setting.len() != 1 {
        return Err(BibErrorResponse::DataDuplicated(0));
    }
    let rental_setting = rental_setting.pop().unwrap();

    if form.borrowed_book_id != "" {
        // Create a DB session
        let mut session = start_transaction(&data)
            .await
            .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

        let ret = borrow_book(
            &db,
            &dbname,
            &cache_map,
            &transaction_map,
            &mut user,
            &form.borrowed_book_id,
            &system_setting.time_zone,
            rental_setting.num_books,
            rental_setting.num_days.into(),
        )
        .await;
        if ret.is_err() {
            // Role back the transaction
            match abort_transaction(&mut session).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e.to_string());
                }
            }
            return Err(ret.unwrap_err());
        }

        // Commit the transaction
        match commit_transaction(&mut session).await {
            Ok(_) => {}
            Err(e) => {
                return Err(BibErrorResponse::SystemError(e.to_string()));
            }
        }
    }

    if form.returned_book_id != "" {
        // Create a DB session
        let mut session = start_transaction(&data)
            .await
            .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
        let ret = unborrow_book(
            &db,
            &dbname,
            &cache_map,
            &transaction_map,
            &mut user,
            &form.returned_book_id,
            &system_setting.time_zone,
        )
        .await;
        if ret.is_err() {
            // Role back the transaction
            match abort_transaction(&mut session).await {
                Ok(_) => {}
                Err(e) => {
                    info!("{}", e.to_string());
                }
            }
            return Err(ret.unwrap_err());
        }

        // Commit the transaction
        match commit_transaction(&mut session).await {
            Ok(_) => {}
            Err(e) => {
                return Err(BibErrorResponse::SystemError(e.to_string()));
            }
        }
    }

    let mut reply = Reply::default();
    reply.user = user.clone();
    for book in user.borrowed_books {
        // Insert the new item at the front to sort in the order of the date
        reply.borrowed_books.insert(0, book.clone());
    }

    Ok(HttpResponse::Ok().json(reply))
}

async fn borrow_book(
    db: &Database,
    dbname: &String,
    cache_map: &web::Data<Mutex<HashMap<String, Cache>>>,
    transaction_map: &web::Data<Mutex<HashMap<String, Transaction>>>,
    user: &mut User,
    book_id: &str,
    time_zone: &str,
    max_borrowing_books: u32,
    max_borrowing_days: i64,
) -> Result<(), BibErrorResponse> {
    // Sanity check
    let locked_cache_map = cache_map.lock().unwrap();
    let cache = locked_cache_map.get(dbname);
    if cache.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    drop(locked_cache_map);

    let num_borrowed_books: u32 = user.borrowed_books.len().try_into().unwrap();
    if num_borrowed_books >= max_borrowing_books {
        return Err(BibErrorResponse::OverBorrowingLimit);
    }

    // Check if the book exists
    let mut book = Book::default();
    let book_id = atoi(book_id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    book.id = book_id;
    let mut books = match search_items(db, &book).await {
        Ok(books) => books,
        Err(_) => {
            return Err(BibErrorResponse::BookNotFound(book.id));
        }
    };
    if books.len() != 1 {
        return Err(BibErrorResponse::DataDuplicated(book.id));
    }

    // Check if the book can be borrowed
    if book.forbidden == "禁帯出" {
        return Err(BibErrorResponse::NotAllowedToBorrow);
    }
    let mut book = books.pop().unwrap();
    {
        let locked_cache_map = cache_map.lock().unwrap();
        let cache = locked_cache_map.get(dbname);
        let borrow_info = cache.unwrap().get(book.id);
        if borrow_info.is_some() {
            info!("book_id({}) is hit in the cached", book_id);
            return Err(BibErrorResponse::BookNotReturned);
        }
    }

    // Increment the transaction counter
    let mut transaction_id;
    {
        let transaction_map = transaction_map.lock().unwrap();
        let transaction = transaction_map.get(dbname);
        if transaction.is_none() {
            return Err(BibErrorResponse::NotAuthorized);
        }
        let transaction = transaction.unwrap();

        let mut counter = transaction.counter.lock().unwrap();
        *counter += 1;
        transaction_id = *counter % (transaction.max_counter + 1);
        if transaction_id == 0 {
            transaction_id = 1;
        }
        *counter = transaction_id;
    }

    let borrowed_book = BorrowedBook::new(
        book_id,
        &book.title,
        get_nowtime(time_zone),
        max_borrowing_days,
        transaction_id,
        book.location.clone(),
    );
    let return_deadline = borrowed_book.return_deadline.clone();
    user.borrowed_books.push(borrowed_book);
    user.borrowed_count += 1;

    // Update the DB
    update_item(db, user)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    book.borrowed_count += 1;
    update_item(db, &book)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    Transaction::borrow(db, transaction_id, user, &book, time_zone)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    // Update the cache
    let locked_cache_map = cache_map.lock().unwrap();
    let cache = locked_cache_map.get(dbname);
    cache.unwrap().borrow(book.id, user.id, return_deadline);

    Ok(())
}

async fn unborrow_book(
    db: &Database,
    dbname: &String,
    cache_map: &web::Data<Mutex<HashMap<String, Cache>>>,
    _transaction_map: &web::Data<Mutex<HashMap<String, Transaction>>>,
    user: &mut User,
    book_id: &str,
    time_zone: &str,
) -> Result<(String, u32), BibErrorResponse> {
    // Sanity check
    let locked_cache_map = cache_map.lock().unwrap();
    let cache = locked_cache_map.get(dbname);
    if cache.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    drop(locked_cache_map);

    // Check if the book exists
    let mut book = Book::default();
    let book_id = atoi(book_id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    book.id = book_id;
    let mut books = match search_items(db, &book).await {
        Ok(books) => books,
        Err(_) => {
            return Err(BibErrorResponse::BookNotFound(book.id));
        }
    };
    if books.len() != 1 {
        return Err(BibErrorResponse::DataDuplicated(book.id));
    }
    book = books.pop().unwrap();

    // Read the user data and check if the book is borrowed
    if user.id == 0 {
        {
            let locked_cache_map = cache_map.lock().unwrap();
            let cache = locked_cache_map.get(dbname);
            let borrow_info = cache.unwrap().get(book.id);
            if borrow_info.is_none() {
                info!("book_id({}) is NOT hit in the cached", book_id);
                return Err(BibErrorResponse::BookNotBorrowed);
            }
            user.id = borrow_info.unwrap().owner_id;
        }
        *user = match search_item(db, user).await {
            Ok(user) => user,
            Err(_) => {
                return Err(BibErrorResponse::UserNotFound(user.id));
            }
        };
    }
    let mut transaction_id: u32 = 0;
    let mut done: bool = false;
    let mut borrowed_date: String = String::new();
    for (pos, borrowed_book) in user.borrowed_books.iter().enumerate() {
        if borrowed_book.book_id == book_id {
            transaction_id = borrowed_book.transaction_id;
            borrowed_date = borrowed_book.borrowed_date.clone();
            user.borrowed_books.remove(pos);
            done = true;
            break;
        }
    }
    if done == false {
        info!("book_id({}) is not hit in the User DB", book_id);
        return Err(BibErrorResponse::BookNotBorrowed);
    }

    // Update the DB
    update_item(db, user)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    Transaction::unborrow(db, transaction_id, user, &book, borrowed_date, time_zone)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    // Update the cache
    let locked_cache_map = cache_map.lock().unwrap();
    let cache = locked_cache_map.get(dbname);
    cache.unwrap().unborrow(book.id);

    Ok((book.title, book.id))
}

fn check_digits_of_user_barcodes(
    setting: &BarcodeSetting,
    data: &str,
) -> Result<(), BibErrorResponse> {
    return check_digits_of_barcodes(setting.user_keta_min, setting.user_keta_max, data);
}

fn check_digits_of_book_barcodes(
    setting: &BarcodeSetting,
    data: &str,
) -> Result<(), BibErrorResponse> {
    return check_digits_of_barcodes(setting.book_keta_min, setting.book_keta_max, data);
}

fn check_digits_of_barcodes(
    keta_min: u32,
    keta_max: u32,
    data: &str,
) -> Result<(), BibErrorResponse> {
    if data == "" {
        return Ok(());
    }

    let len: u32 = data
        .len()
        .try_into()
        .map_err(|_| BibErrorResponse::InvalidArgument(data.to_owned()))?;

    if keta_min <= len && len <= keta_max {
        return Ok(());
    } else {
        return Err(BibErrorResponse::BarcodeDigitsOutOfRange);
    }
}
