use crate::db_client::*;
use crate::error::*;
use crate::item::atoi;
use crate::item::RentalSetting;
use crate::item::{search_item, search_items, update_item};
use crate::item::{Book, BorrowedBook, User};
use crate::views::cache::*;
use crate::views::reply::Reply;
use crate::views::session::*;
use crate::views::transaction::*;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use log::{debug, error};
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub user_id: String,
    pub borrowed_book_id: String,
    pub returned_book_id: String,
}

pub async fn process(
    session: Session,
    form: web::Form<FormData>,
    data: web::Data<Mutex<DbClient>>,
    cache: web::Data<Cache>,
    transaction: web::Data<Transaction>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    check_session(&session)?;
    let db = get_db(&data).await?;

    let mut user = User::default();
    if form.user_id == "" && form.borrowed_book_id == "" && form.returned_book_id != "" {
        let (book_title, book_id) =
            unborrow_book(&db, &cache, &transaction, &mut user, &form.returned_book_id).await?;
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
            disconnect_db(&data);
            return Err(BibErrorResponse::UserNotFound(user.id));
        }
    };

    let mut setting = RentalSetting::default();
    setting.id = 1;
    let mut setting = match search_items(&db, &setting).await {
        Ok(setting) => setting,
        Err(e) => {
            disconnect_db(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    if setting.len() != 1 {
        return Err(BibErrorResponse::DataDuplicated);
    }
    let setting = setting.pop().unwrap();

    if form.borrowed_book_id != "" {
        borrow_book(
            &db,
            &cache,
            &transaction,
            &mut user,
            &form.borrowed_book_id,
            setting.num_books,
            setting.num_days.into(),
        )
        .await?;
    }

    if form.returned_book_id != "" {
        unborrow_book(&db, &cache, &transaction, &mut user, &form.returned_book_id).await?;
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
    db: &DbInstance,
    cache: &web::Data<Cache>,
    transaction: &web::Data<Transaction>,
    user: &mut User,
    book_id: &str,
    max_borrowing_books: u32,
    max_borrowing_days: i64,
) -> Result<(), BibErrorResponse> {
    let num_borrowed_books: u32 = user.borrowed_books.len().try_into().unwrap();
    if num_borrowed_books >= max_borrowing_books {
        return Err(BibErrorResponse::OverBorrowingLimit);
    }

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
        return Err(BibErrorResponse::DataDuplicated);
    }
    let mut book = books.pop().unwrap();
    let borrow_info = cache.get(book.id);
    if borrow_info.is_some() {
        return Err(BibErrorResponse::BookNotReturned);
    }

    let mut counter = transaction.counter.lock().unwrap();
    *counter += 1;
    let transaction_id = *counter % transaction.max_counter;
    drop(counter);

    let borrowed_book = BorrowedBook::new(book_id, &book.title, max_borrowing_days, transaction_id);
    let return_deadline = borrowed_book.return_deadline.clone();
    user.borrowed_books.push(borrowed_book);
    user.borrowed_count += 1;
    update_item(db, user)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    // Check the returned data
    let mut done: bool = false;
    for book in &user.borrowed_books {
        if book_id == book.book_id {
            debug!("Check passed, book_id = {}", book_id);
            done = true;
            break;
        }
    }
    if !done {
        return Err(BibErrorResponse::SystemError("Check failed".to_string()));
    }

    cache.borrow(book.id, user.id, return_deadline);

    // Don't propagate error
    book.borrowed_count += 1;
    if let Err(e) = update_item(db, &book).await {
        error!("{:?}", e);
    }

    debug!("transaction_id = {}", transaction_id);
    Transaction::borrow(db, transaction_id, user, &book)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))
}

async fn unborrow_book(
    db: &DbInstance,
    cache: &web::Data<Cache>,
    _transaction: &web::Data<Transaction>,
    user: &mut User,
    book_id: &str,
) -> Result<(String, u32), BibErrorResponse> {
    debug!("unborrow_book,id = {}", book_id);

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
        return Err(BibErrorResponse::DataDuplicated);
    }
    book = books.pop().unwrap();

    if user.id == 0 {
        let borrow_info = cache.get(book.id);
        if borrow_info.is_none() {
            return Err(BibErrorResponse::BookNotBorrowed);
        }
        user.id = borrow_info.unwrap().owner_id;
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
        return Err(BibErrorResponse::BookNotBorrowed);
    }

    update_item(db, user)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    // Check the returned data
    let mut done: bool = true;
    for book in &user.borrowed_books {
        if book_id == book.book_id {
            error!("Check failed, book_id = {}", book_id);
            done = false;
            break;
        }
    }
    if !done {
        return Err(BibErrorResponse::SystemError("Check failed".to_string()));
    }
    debug!("Check passed, book_id = {}", book_id);

    cache.unborrow(book.id);

    debug!("transaction_id = {}", transaction_id);
    Transaction::unborrow(db, transaction_id, user, &book, borrowed_date)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
    Ok((book.title, book.id))
}
