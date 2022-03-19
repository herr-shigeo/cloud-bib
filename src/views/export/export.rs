use crate::db_client::*;
use crate::error::BibErrorResponse;
use crate::item::TransactionItem;
use crate::item::{search_items, Book, User};
use crate::views::content_loader::read_file;
use crate::views::session::check_session;
use crate::Transaction;
use actix_files::NamedFile;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use chrono::{TimeZone, Utc};
use chrono_tz::Europe::Berlin;
use csv::WriterBuilder;
use log::error;
use std::error;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

pub async fn load(_session: Session) -> HttpResponse {
    let html_data = read_file("src/html/export.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

fn write_user_list(users: Vec<User>) -> Result<String, Box<dyn error::Error>> {
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);

    for user in users {
        wtr.serialize(User {
            id: user.id,
            name: user.name.clone(),
            kana: user.kana.clone(),
            category: user.category.clone(),
            remark: user.remark.clone(),
            register_date: user.register_date.clone(),
            borrowed_count: user.borrowed_count.clone(),
            reserved: user.reserved.clone(),
            borrowed_books: vec![],
        })?;
    }

    let utc = Utc::now().naive_utc();
    let dt = Berlin.from_utc_datetime(&utc);
    let fname = format!("user_list_{}.csv", dt.format("%Y%m%d"));
    let mut file = File::create(fname.clone())?;
    file.write_all(&wtr.into_inner()?)?;

    Ok(fname)
}

pub async fn export_user_list(
    session: Session,
    data: web::Data<Mutex<DbClient>>,
) -> Result<NamedFile, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data).await?;

    let user = User::default();
    let users = match search_items(&db, &user).await {
        Ok(users) => users,
        Err(e) => {
            disconnect_db(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    match write_user_list(users) {
        Ok(fname) => {
            return Ok(
                NamedFile::open(fname).map_err(|e| BibErrorResponse::SystemError(e.to_string()))?
            );
        }
        Err(e) => {
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };
}

fn write_book_list(books: Vec<Book>) -> Result<String, Box<dyn error::Error>> {
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);

    for book in books {
        wtr.serialize(Book {
            id: book.id,
            title: book.title.clone(),
            kana: book.kana.clone(),
            char: book.char.clone(),
            recommendation: book.recommendation.clone(),
            status: book.status.clone(),
            register_type: book.register_type.clone(),
            author: book.author.clone(),
            publisher: book.publisher.clone(),
            series: book.series.clone(),
            remark: book.remark.clone(),
            register_date: book.register_date.clone(),
            borrowed_count: book.borrowed_count.clone(),
            reserved: book.reserved.clone(),
            owner_id: None,
            return_deadline: None,
        })?;
    }

    let utc = Utc::now().naive_utc();
    let dt = Berlin.from_utc_datetime(&utc);
    let fname = format!("book_list_{}.csv", dt.format("%Y%m%d"));
    let mut file = File::create(fname.clone())?;
    file.write_all(&wtr.into_inner()?)?;

    Ok(fname)
}

pub async fn export_book_list(
    session: Session,
    data: web::Data<Mutex<DbClient>>,
) -> Result<NamedFile, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data).await?;

    let book = Book::default();
    let books = match search_items(&db, &book).await {
        Ok(books) => books,
        Err(e) => {
            error!("{:?}", e);
            disconnect_db(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    match write_book_list(books) {
        Ok(fname) => {
            return Ok(
                NamedFile::open(fname).map_err(|e| BibErrorResponse::SystemError(e.to_string()))?
            );
        }
        Err(e) => {
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };
}

fn write_transaction_list(items: Vec<TransactionItem>) -> Result<String, Box<dyn error::Error>> {
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);

    for item in items {
        wtr.serialize(TransactionItem {
            id: item.id,
            user_id: item.user_id,
            user_name: item.user_name,
            book_id: item.book_id,
            book_title: item.book_title,
            borrowed_date: item.borrowed_date,
            returned_date: item.returned_date,
        })?;
    }

    let utc = Utc::now().naive_utc();
    let dt = Berlin.from_utc_datetime(&utc);
    let fname = format!("transaction_list_{}.csv", dt.format("%Y%m%d"));
    let mut file = File::create(fname.clone())?;
    file.write_all(&wtr.into_inner()?)?;

    Ok(fname)
}

pub async fn export_history_list(
    session: Session,
    data: web::Data<Mutex<DbClient>>,
) -> Result<NamedFile, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data).await?;

    let item = TransactionItem::default();
    let transaction_items = Transaction::search(&db, &item).await;

    if transaction_items.len() == 0 {
        return Err(BibErrorResponse::DataNotFound(String::new()));
    }

    match write_transaction_list(transaction_items) {
        Ok(fname) => {
            return Ok(
                NamedFile::open(fname).map_err(|e| BibErrorResponse::SystemError(e.to_string()))?
            );
        }
        Err(e) => {
            error!("{:?}", e);
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };
}
