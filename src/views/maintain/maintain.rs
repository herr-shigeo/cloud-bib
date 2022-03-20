use crate::db_client::*;
use crate::error::*;
use crate::item::{delete_item_all, TransactionItem};
use crate::views::content_loader::read_file;
use crate::views::reply::Reply;
use crate::views::session::check_session;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
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
    _data: web::Data<Mutex<DbClient>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_session(&session)?;
    //let db = get_db(&data).await?;

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}
