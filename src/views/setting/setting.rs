use crate::error::*;
use crate::item::{insert_item, search_items, update_item, Book, SystemSetting, User};
use crate::item::{RentalSetting, SystemUser};
use crate::views::content_loader::read_csv;
use crate::views::content_loader::read_file;
use crate::views::reply::Reply;
use crate::views::session::{check_session, get_string_value};
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use futures::{StreamExt, TryStreamExt};
use log::{debug, error, info};
use serde::Deserialize;
use shared_mongodb::{database, ClientHolder};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;
extern crate sanitize_filename;
use crate::views::db_helper::{get_db, get_db_with_name};
use std::io::{Error, ErrorKind};
use std::{env, error};

use lazy_static::lazy_static;

lazy_static! {
    static ref DB_COMMON_NAME: String =
        env::var("BIB_DB_NAME").expect("You must set the BIB_DB_NAME environment var!");
}

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
    pub password: String,
}

pub async fn update_rental_setting(
    session: Session,
    form: web::Form<Form1Data>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    check_session(&session)?;
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

pub async fn get_rental_setting(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data, &session).await?;

    let mut setting = RentalSetting::default();
    setting.id = 1;
    let mut setting = match search_items(&db, &setting).await {
        Ok(setting) => setting,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    if setting.len() != 1 {
        return Err(BibErrorResponse::DataDuplicated);
    }
    let setting = setting.pop().unwrap();

    let mut reply = Reply::default();
    reply.num_books = setting.num_books;
    reply.num_days = setting.num_days;

    Ok(HttpResponse::Ok().json(reply))
}

pub async fn update_system_setting(
    session: Session,
    form: web::Form<Form2Data>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db_with_name(&data, &DB_COMMON_NAME).await?;

    let mut setting = SystemUser::default();
    setting.password = form.password.clone();
    setting.dbname = get_string_value(&session, "dbname")?;

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
    _setting_map: web::Data<HashMap<String, SystemSetting>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data, &session).await?;

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

    // Check the number of items(TODO)

    let mut users = vec![];
    for i in 0..records.len() {
        let record = &records[i];
        let num_field = record.len();
        if num_field != 6 {
            return Err(BibErrorResponse::InvalidArgument(num_field.to_string()));
        }
        debug!(
            "{}, {}, {}, {}, {}, {}",
            &record[0], &record[1], &record[2], &record[3], &record[4], &record[5]
        );
        let user = User::new(
            &record[0], &record[1], &record[2], &record[3], &record[4], &record[5],
        )
        .map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
        users.push(user);
    }
    for user in users {
        if let Err(e) = insert_item(&db, &user).await {
            database::disconnect(&data);
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

pub async fn import_book_list(
    session: Session,
    payload: Multipart,
    data: web::Data<Mutex<ClientHolder>>,
    _setting_map: web::Data<HashMap<String, SystemSetting>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data, &session).await?;

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

    // Check the number of items(TODO)

    let mut books = vec![];
    for i in 0..records.len() {
        let record = &records[i];
        let num_field = record.len();
        if num_field != 12 {
            error!("Invalid field num = {}", num_field);
            return Err(BibErrorResponse::InvalidArgument(num_field.to_string()));
        }
        debug!(
            "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
            &record[0],  // id
            &record[1],  // title
            &record[2],  // char
            &record[3],  // register_type
            &record[4],  // recommendation
            &record[5],  // remark
            &record[6],  // status
            &record[7],  // author
            &record[8],  // publisher
            &record[9],  // series
            &record[10], // kana
            &record[11], // register_date
        );
        let book = Book::new(
            &record[0],  // id
            &record[1],  // title
            &record[10], // kana
            &record[9],  // series
            &record[7],  // author
            &record[8],  // publisher
            &record[2],  // char
            &record[5],  // remark
            &record[4],  // recommendation
            &record[11], // register_date
            &record[3],  // register_type
            &record[6],  // status
        )
        .map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
        books.push(book);
    }
    for book in books {
        if let Err(e) = insert_item(&db, &book).await {
            error!("{:?}", e);
            database::disconnect(&data);
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    }

    debug!("Done");
    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}
