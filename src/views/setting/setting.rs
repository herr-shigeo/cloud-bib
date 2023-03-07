use crate::error::*;
use crate::item::{insert_item, search_item, search_items, update_item, Book, SystemSetting, User};
use crate::item::{BarcodeSetting, RentalSetting};
use crate::views::content_loader::read_csv;
use crate::views::content_loader::read_file;
use crate::views::reply::Reply;
use crate::views::session::check_operator_session;
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use futures::{StreamExt, TryStreamExt};
use log::{debug, error, info};
use mongodb::Database;
use serde::Deserialize;
use shared_mongodb::{database, ClientHolder};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;
extern crate sanitize_filename;
use crate::views::db_helper::get_db;
use futures::future::join_all;
use std::error;
use std::io::{Error, ErrorKind};

pub async fn load() -> HttpResponse {
    let html_data = read_file("src/html/setting.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

#[derive(Deserialize, Debug)]
pub struct Form1Data {
    pub num_books: String,
    pub num_days: String,
}

#[derive(Deserialize, Debug)]
pub struct Form2Data {
    pub user_keta_min: String,
    pub user_keta_max: String,
    pub book_keta_min: String,
    pub book_keta_max: String,
}

pub async fn update_rental_setting(
    session: Session,
    form: web::Json<Form1Data>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let mut setting = match RentalSetting::new(&form.num_books, &form.num_days) {
        Ok(setting) => setting,
        Err(e) => {
            return Err(BibErrorResponse::InvalidArgument(e.to_string()));
        }
    };
    setting.id = 1;

    match update_item(&db, &setting).await {
        Ok(setting) => setting,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

pub async fn get_setting(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let mut rental_setting = RentalSetting::default();
    rental_setting.id = 1;
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

    let mut barcode_setting = BarcodeSetting::default();
    barcode_setting.id = 1;
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

    let mut reply = Reply::default();
    reply.rental_setting = rental_setting;
    reply.barcode_setting = barcode_setting;

    Ok(HttpResponse::Ok().json(reply))
}

pub async fn update_barcode_setting(
    session: Session,
    form: web::Json<Form2Data>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let setting = match BarcodeSetting::new(
        &form.user_keta_min,
        &form.user_keta_max,
        &form.book_keta_min,
        &form.book_keta_max,
    ) {
        Ok(setting) => setting,
        Err(e) => {
            return Err(BibErrorResponse::InvalidArgument(e.to_string()));
        }
    };

    match update_item(&db, &setting).await {
        Ok(setting) => setting,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

async fn save_file(mut payload: Multipart) -> Result<String, Box<dyn error::Error>> {
    if let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().ok_or("content_type error")?;
        let filename = content_type.get_filename().ok_or("filename error")?;
        let file_path = format!("/tmp/{}", sanitize_filename::sanitize(&filename));
        info!("{}", file_path);
        let returned_file_path = file_path.clone();

        let mut f = web::block(|| std::fs::File::create(file_path)).await?;

        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
        return Ok(returned_file_path);
    }
    Err(Box::new(Error::new(ErrorKind::Other, "Playload not found")))
}

pub async fn import_user_list(
    session: Session,
    payload: Multipart,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<Mutex<HashMap<String, SystemSetting>>>,
) -> Result<HttpResponse, BibErrorResponse> {
    let dbname = check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let setting_map = setting_map.lock().unwrap();
    let setting = setting_map.get(&dbname);
    if setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let setting = setting.unwrap().clone();
    drop(setting_map);

    let file_path = match save_file(payload).await {
        Ok(file_path) => file_path,
        Err(e) => {
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };

    let records = match read_csv(&file_path) {
        Ok(records) => records,
        Err(e) => {
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };

    // Check number of items that can be registered at once
    let nrecords: u32 = records.len().try_into().unwrap();
    if nrecords > setting.max_parallel_registrations {
        return Err(BibErrorResponse::ExceedLimitInParallel(
            setting.max_parallel_registrations,
        ));
    }

    // Check the number of items
    let mut user = User::default();
    user.id = 0;
    let users = match search_items(&db, &user).await {
        Ok(users) => users,
        Err(_) => vec![],
    };
    let nusers: u32 = users.len().try_into().unwrap();
    let nsize = nusers + nrecords;
    if nsize > setting.max_registered_users {
        return Err(BibErrorResponse::ExceedLimit(nsize));
    }

    // Check the paramters
    let mut map: HashMap<u32, bool> = HashMap::new();
    let mut users = vec![];
    for i in 0..records.len() {
        let record = &records[i];
        let num_field = record.len();
        if num_field != 7 {
            return Err(BibErrorResponse::InvalidArgument(format!(
                "The number of fields is {}",
                num_field
            )));
        }
        debug!("{:?}", record);
        let user = User::new(
            &record[0], &record[1], &record[2], &record[3], &record[4], &record[5], &record[6],
        )
        .map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
        if map.insert(user.id, true).is_some() {
            return Err(BibErrorResponse::DataDuplicated(user.id));
        }
        match search_item(&db, &user).await {
            Ok(user) => {
                return Err(BibErrorResponse::DataDuplicated(user.id));
            }
            Err(_) => {}
        };
        users.push(user);
    }

    // Update the DB
    let mut num_processed: usize = 0;
    loop {
        let mut futures = vec![];
        loop {
            if futures.len() as u32 == setting.num_threads || num_processed as u32 == nrecords {
                break;
            }
            futures.push(insert_item(&db, &users[num_processed]));
            num_processed += 1;
        }
        let reses = join_all(futures).await;
        for res in reses {
            match res {
                Err(e) => {
                    database::disconnect(&data);
                    return Err(BibErrorResponse::SystemError(e.to_string()));
                }
                Ok(_) => {}
            }
        }
        if num_processed as u32 == nrecords {
            break;
        }
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

async fn check_item(db: &Database, id: u32) -> Result<(), BibErrorResponse> {
    let mut book = Book::default();
    book.id = id;

    match search_item(&db, &book).await {
        Ok(book) => Err(BibErrorResponse::DataDuplicated(book.id)),
        Err(_) => Ok(()),
    }
}

pub async fn import_book_list(
    session: Session,
    payload: Multipart,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<Mutex<HashMap<String, SystemSetting>>>,
) -> Result<HttpResponse, BibErrorResponse> {
    let dbname = check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let setting_map = setting_map.lock().unwrap();
    let setting = setting_map.get(&dbname);
    if setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let setting = setting.unwrap().clone();
    drop(setting_map);

    let file_path = match save_file(payload).await {
        Ok(file_path) => file_path,
        Err(e) => {
            error!("{:?}", e);
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };

    let records = match read_csv(&file_path) {
        Ok(records) => records,
        Err(e) => {
            error!("{:?}", e);
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };

    // Check the number of items that can be registered at once
    let nrecords: u32 = records.len().try_into().unwrap();
    if nrecords > setting.max_parallel_registrations {
        return Err(BibErrorResponse::ExceedLimitInParallel(
            setting.max_parallel_registrations,
        ));
    }

    // Check the number of items
    let mut book = Book::default();
    book.id = 0;
    let books = match search_items(&db, &book).await {
        Ok(books) => books,
        Err(_) => vec![],
    };
    let nbooks: u32 = books.len().try_into().unwrap();
    let nsize = nbooks + nrecords;
    if nsize > setting.max_registered_books {
        return Err(BibErrorResponse::ExceedLimit(nsize));
    }

    // Check the parameter
    let mut map: HashMap<u32, bool> = HashMap::new();
    let mut books = vec![];
    let num_books = records.len();
    let mut num_processed: usize = 0;

    loop {
        let mut futures = vec![];
        loop {
            if futures.len() as u32 == setting.num_threads || num_processed == num_books {
                break;
            }
            let record = &records[num_processed];
            let num_field = record.len();
            if num_field != 20 {
                return Err(BibErrorResponse::InvalidArgument(format!(
                    "The number of fields is {}",
                    num_field
                )));
            }
            debug!("{:?}", record);
            let book = Book::new(
                &record[0],  // id
                &record[1],  // title
                &record[2],  // location
                &record[3],  // category
                &record[4],  // status
                &record[5],  // author
                &record[6],  // publisher
                &record[7],  // published_date
                &record[8],  // series
                &record[9],  // volume
                &record[10], // page
                &record[11], // kana
                &record[12], // category_symbol
                &record[13], // library_symbol
                &record[14], // volume_symbol
                &record[15], // forbidden
                &record[16], // remark
                &record[17], // isbn
                &record[18], // register_date
                &record[19], // register_type
            )
            .map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
            if map.insert(book.id, true).is_some() {
                return Err(BibErrorResponse::DataDuplicated(book.id));
            }
            futures.push(check_item(&db, book.id));
            num_processed += 1;
            books.push(book);
        }
        let reses = join_all(futures).await;
        for res in reses {
            res?;
        }
        if num_processed == num_books {
            break;
        }
    }

    // Update the DB
    num_processed = 0;
    loop {
        let mut futures = vec![];
        loop {
            if futures.len() as u32 == setting.num_threads || num_processed as u32 == nrecords {
                break;
            }
            futures.push(insert_item(&db, &books[num_processed]));
            num_processed += 1;
        }
        let reses = join_all(futures).await;
        for res in reses {
            match res {
                Err(e) => {
                    database::disconnect(&data);
                    return Err(BibErrorResponse::SystemError(e.to_string()));
                }
                Ok(_) => {}
            }
        }
        if num_processed as u32 == nrecords {
            break;
        }
    }

    debug!("Done");
    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}
