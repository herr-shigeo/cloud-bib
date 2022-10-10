use crate::error::BibErrorResponse;
use crate::item::atoi;
use crate::item::search_items;
use crate::item::Book;
use crate::views::cache::*;
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::views::session::check_any_session;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use shared_mongodb::ClientHolder;
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub id: String,
    pub title: String,
    pub kana: String,
    pub author: String,
}

#[derive(Serialize, Debug)]
pub struct BookList {
    pub books: Vec<Book>,
}

pub async fn search_book(
    session: Session,
    form: web::Query<FormData>,
    data: web::Data<Mutex<ClientHolder>>,
    cache: web::Data<Cache>,
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
    book.author = form.author.clone();
    get_book_list(&session, data, &cache, &book).await
}

async fn get_book_list(
    session: &Session,
    data: web::Data<Mutex<ClientHolder>>,
    cache: &web::Data<Cache>,
    book: &Book,
) -> Result<HttpResponse, BibErrorResponse> {
    let db = get_db(&data, Some(session)).await?;

    let mut books = match search_items(&db, book).await {
        Ok(books) => books,
        Err(_) => {
            return Err(BibErrorResponse::BookNotFound(book.id));
        }
    };

    for mut book in &mut books {
        if let Some(info) = cache.get(book.id) {
            book.owner_id = Some(info.owner_id);
            book.return_deadline = Some(info.return_deadline.clone());
        }
    }

    let mut reply = Reply::default();
    reply.book_list.append(&mut books);

    Ok(HttpResponse::Ok().json(reply))
}
