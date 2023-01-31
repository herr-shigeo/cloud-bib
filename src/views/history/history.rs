use crate::error::BibErrorResponse;
use crate::item::atoi;
use crate::views::content_loader::read_file;
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::Transaction;
use crate::TransactionItem;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use log::debug;
use serde::Deserialize;
use shared_mongodb::ClientHolder;
use std::sync::Mutex;

pub async fn load(_session: Session) -> HttpResponse {
    let html_data = read_file("src/html/history.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub user_id: String,
    pub user_name: String,
    pub book_id: String,
    pub book_title: String,
}

pub async fn search(
    session: Session,
    form: web::Query<FormData>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    let db = get_db(&data, &session).await?;

    let mut user_id = 0;
    if form.user_id != "" {
        user_id =
            atoi(&form.user_id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    }

    let mut book_id = 0;
    if form.book_id != "" {
        book_id =
            atoi(&form.book_id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    }

    let item = TransactionItem::new(user_id, &form.user_name, book_id, &form.book_title);
    let mut transaction_items = Transaction::search(&db, &item).await;

    let mut reply = Reply::default();
    reply.transaction_list.append(&mut transaction_items);
    Ok(HttpResponse::Ok().json(reply))
}
