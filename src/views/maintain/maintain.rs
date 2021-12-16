use crate::db_client::*;
use crate::error::*;
use crate::item::{delete_item_all, search_items, update_item, TransactionItem, User};
use crate::item::{RentalSetting, SystemSetting};
use crate::views::content_loader::read_file;
use crate::views::reply::Reply;
use crate::views::session::check_session;
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use futures::{StreamExt, TryStreamExt};
use log::{debug, info};
use std::error;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::sync::Mutex;

pub async fn load() -> HttpResponse {
    let html_data = read_file("src/html/maintain.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

pub async fn clear_status(
    session: Session,
    data: web::Data<Mutex<DbClient>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data).await?;

    let item = TransactionItem::default();
    delete_item_all(&db, &item)
        .await
        .map_err(|e| BibErrorResponse::DataNotFound(e.to_string()))?;

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

pub async fn clear_history(
    session: Session,
    data: web::Data<Mutex<DbClient>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data).await?;

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}
