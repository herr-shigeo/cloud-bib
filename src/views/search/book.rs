use crate::error::BibErrorResponse;
use crate::item::atoi;
use crate::item::search_items;
use crate::item::Book;
use crate::views::cache::*;
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::views::session::check_operator_session;
use crate::views::session::check_user_session;
use crate::views::utils::fetch_book_info;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use futures::TryFutureExt;
use log::debug;
use serde::{Deserialize, Serialize};
use shared_mongodb::ClientHolder;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub id: String,
    pub title: String,
    pub kana: String,
    pub author: String,
    pub user_id: String,
}

#[derive(Deserialize, Debug)]
pub struct Form2Data {
    pub isbn: String,
}

#[derive(Serialize, Debug)]
pub struct BookList {
    pub books: Vec<Book>,
}

pub async fn search_isbn(
    _session: Session,
    form: web::Query<Form2Data>,
) -> Result<HttpResponse, BibErrorResponse> {
    let isbn = form.isbn.clone();

    let fut = async move {
        fetch_book_info(&isbn)
            .await
            .map_err(|_| BibErrorResponse::DataNotFound("isbn".to_string()))
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let book = rt.block_on(fut)?;

    let mut reply = Reply::default();
    reply.book_list = vec![book];

    Ok(HttpResponse::Ok().json(reply))
}

pub async fn search_book(
    session: Session,
    form: web::Query<FormData>,
    data: web::Data<Mutex<ClientHolder>>,
    cache_map: web::Data<Mutex<HashMap<String, Cache>>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);

    let user_id = form
        .user_id
        .parse()
        .map_err(|_db| BibErrorResponse::InvalidArgument(form.user_id.to_owned()))?;
    let dbname;
    if user_id == 0 {
        dbname = check_operator_session(&session)?;
    } else {
        dbname = check_user_session(&session, user_id)?;
    }

    let cache_map = cache_map.lock().unwrap();
    let cache = cache_map.get(&dbname);
    if cache.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let cache = cache.unwrap();

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
    cache: &Cache,
    book: &Book,
) -> Result<HttpResponse, BibErrorResponse> {
    let db = get_db(&data, session).await?;

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
