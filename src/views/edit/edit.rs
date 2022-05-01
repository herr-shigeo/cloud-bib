use crate::db_client::*;
use crate::error::BibErrorResponse;
use crate::item::atoi;
use crate::item::{delete_item, search_item, update_item};
use crate::item::{Book, User};
use crate::views::content_loader::read_file;
use crate::views::reply::Reply;
use crate::views::session::check_session;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use log::debug;
use serde::Deserialize;
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
    data: web::Data<Mutex<DbClient>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    check_session(&session)?;
    let db = get_db(&data).await?;

    // Read the User from DB first
    let mut user = User::default();
    user.id = atoi(&form.user_id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    user = search_item(&db, &user)
        .await
        .map_err(|_| BibErrorResponse::UserNotFound(user.id))?;
    user.name = form.user_name.clone();
    user.kana = form.user_kana.clone();
    user.category = form.user_category.clone();
    user.remark = form.user_remark.clone();
    user.register_date = form.user_register_date.clone();

    let operation: &str = &form.operation;
    match operation {
        "update" => {
            update_item(&db, &user)
                .await
                .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
        }
        "delete" => {
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
    data: web::Data<Mutex<DbClient>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    check_session(&session)?;
    let db = get_db(&data).await?;

    // Read the Book from DB first
    let mut book = Book::default();
    book.id = atoi(&form.book_id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    book = search_item(&db, &book)
        .await
        .map_err(|_| BibErrorResponse::BookNotFound(book.id))?;
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

    let operation: &str = &form.operation;
    match operation {
        "update" => {
            update_item(&db, &book)
                .await
                .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
        }
        "delete" => {
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
