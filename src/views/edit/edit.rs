use crate::error::BibErrorResponse;
use crate::item::{atoi, search_items, SystemSetting};
use crate::item::{delete_item, search_item, update_item};
use crate::item::{Book, User};
use crate::views::cache::Cache;
use crate::views::content_loader::read_file;
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::views::session::{check_session, get_string_value};
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use log::debug;
use serde::Deserialize;
use shared_mongodb::ClientHolder;
use std::collections::HashMap;
use std::sync::Mutex;

pub async fn load(_session: Session) -> HttpResponse {
    let html_data = read_file("src/html/edit.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

#[derive(Deserialize, Debug)]
pub struct Form1Data {
    pub user_id: String,
    pub user_name: String,
    pub user_kana: String,
    pub user_category: String,
    pub user_remark: String,
    pub operation: String,
    pub user_register_date: String,
}

pub async fn user(
    session: Session,
    form: web::Form<Form1Data>,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<HashMap<String, SystemSetting>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    check_session(&session)?;
    let db = get_db(&data, &session).await?;
    let dbname = get_string_value(&session, "dbname")?;
    let setting = setting_map.get(&dbname);
    if setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let setting = setting.unwrap();

    // Read the User from DB first
    let mut user = User::default();
    user.id = atoi(&form.user_id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    user = match search_item(&db, &user).await {
        Ok(mut user) => {
            user.name = form.user_name.clone();
            user.kana = form.user_kana.clone();
            user.category = form.user_category.clone();
            user.remark = form.user_remark.clone();
            user.register_date = form.user_register_date.clone();
            user
        }
        Err(_) => {
            // Check the number of items
            user.id = 0;
            let users = match search_items(&db, &user).await {
                Ok(users) => users,
                Err(_) => {
                    return Err(BibErrorResponse::UserNotFound(user.id));
                }
            };
            let nsize: u32 = users.len().try_into().unwrap();
            if nsize == setting.max_registered_users {
                return Err(BibErrorResponse::ExceedLimit(nsize));
            }

            User::new(
                &form.user_id,
                &form.user_name,
                &form.user_kana,
                &form.user_category,
                &form.user_remark,
                &form.user_register_date,
            )
            .unwrap()
        }
    };

    let operation: &str = &form.operation;
    match operation {
        "update" => {
            update_item(&db, &user)
                .await
                .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
        }
        "delete" => {
            if user.borrowed_books.len() > 0 {
                return Err(BibErrorResponse::NotPossibleToDelete);
            }
            delete_item(&db, &user)
                .await
                .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
        }
        _ => {
            return Err(BibErrorResponse::NotImplemented);
        }
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

#[derive(Deserialize, Debug)]
pub struct Form2Data {
    pub book_id: String,
    pub book_title: String,
    pub book_kana: String,
    pub book_series: String,
    pub book_author: String,
    pub book_publisher: String,
    pub book_char: String,
    pub book_remark: String,
    pub book_recommendation: String,
    pub book_register_type: String,
    pub book_register_date: String,
    pub book_status: String,
    pub operation: String,
}

pub async fn book(
    session: Session,
    form: web::Form<Form2Data>,
    data: web::Data<Mutex<ClientHolder>>,
    cache_map: web::Data<HashMap<String, Cache>>,
    setting_map: web::Data<HashMap<String, SystemSetting>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    check_session(&session)?;
    let db = get_db(&data, &session).await?;
    let dbname = get_string_value(&session, "dbname")?;
    let setting = setting_map.get(&dbname);
    if setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let setting = setting.unwrap();

    let cache = cache_map.get(&dbname);
    if cache.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let cache = cache.unwrap();

    // Read the Book from DB first
    let mut book = Book::default();
    book.id = atoi(&form.book_id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    book = match search_item(&db, &book).await {
        Ok(mut book) => {
            book.title = form.book_title.clone();
            book.kana = form.book_kana.clone();
            book.series = form.book_series.clone();
            book.author = form.book_author.clone();
            book.publisher = form.book_publisher.clone();
            book.char = form.book_char.clone();
            book.remark = form.book_remark.clone();
            book.recommendation = form.book_recommendation.clone();
            book.register_date = form.book_register_date.clone();
            book.register_type = form.book_register_type.clone();
            book.status = form.book_status.clone();
            book
        }
        Err(_) => {
            // Check the number of items
            book.id = 0;
            let books = match search_items(&db, &book).await {
                Ok(books) => books,
                Err(_) => {
                    return Err(BibErrorResponse::BookNotFound(book.id));
                }
            };
            let nsize: u32 = books.len().try_into().unwrap();
            if nsize == setting.max_registered_users {
                return Err(BibErrorResponse::ExceedLimit(nsize));
            }

            Book::new(
                &form.book_id,
                &form.book_title,
                &form.book_kana,
                &form.book_series,
                &form.book_author,
                &form.book_publisher,
                &form.book_char,
                &form.book_remark,
                &form.book_recommendation,
                &form.book_register_date,
                &form.book_register_type,
                &form.book_status,
            )
            .unwrap()
        }
    };

    let operation: &str = &form.operation;
    match operation {
        "update" => {
            update_item(&db, &book)
                .await
                .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
        }
        "delete" => {
            if cache.get(book.id).is_some() {
                return Err(BibErrorResponse::NotPossibleToDelete);
            }
            delete_item(&db, &book)
                .await
                .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
        }
        _ => {
            return Err(BibErrorResponse::NotImplemented);
        }
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}
