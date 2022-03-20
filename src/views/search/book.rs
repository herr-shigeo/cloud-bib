use crate::db_client::*;
use crate::error::BibErrorResponse;
use crate::item::atoi;
use crate::item::search_items;
use crate::item::Book;
use crate::views::reply::Reply;
use crate::views::session::check_any_session;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub id: String,
    pub title: String,
    pub kana: String,
}

#[derive(Serialize, Debug)]
pub struct BookList {
    pub books: Vec<Book>,
}

pub async fn search_book(
    session: Session,
    form: web::Query<FormData>,
    data: web::Data<Mutex<DbClient>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    check_any_session(&session)?;

    let mut book = Book::default();
    if form.id == "" {
        book.id = 0;
    } else {
        book.id = match atoi(&form.id) {
            Ok(id) => id,
            Err(e) => {
                return Err(BibErrorResponse::InvalidArgument(e.to_string()));
            }
        };
    }
    book.title = form.title.clone();
    book.kana = form.kana.clone();
    get_book_list(data, &book).await
}

async fn get_book_list(
    data: web::Data<Mutex<DbClient>>,
    book: &Book,
) -> Result<HttpResponse, BibErrorResponse> {
    let db = get_db(&data).await?;

    let mut books = match search_items(&db, book).await {
        Ok(books) => books,
        Err(_) => {
            return Err(BibErrorResponse::BookNotFound(book.id));
        }
    };

    let mut reply = Reply::default();
    reply.book_list.append(&mut books);

    Ok(HttpResponse::Ok().json(reply))
}
