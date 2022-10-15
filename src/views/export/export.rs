use crate::error::BibErrorResponse;
use crate::item::{search_items, Book, User};
use crate::item::{SystemSetting, TransactionItem};
use crate::views::content_loader::read_file;
use crate::views::db_helper::get_db;
use crate::views::session::{check_session, get_string_value};
use crate::views::utils::get_nowtime;
use crate::Transaction;
use actix_files::NamedFile;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use csv::WriterBuilder;
use log::error;
use shared_mongodb::{database, ClientHolder};
use std::collections::HashMap;
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

fn write_user_list(users: Vec<User>, time_zone: &str) -> Result<String, Box<dyn error::Error>> {
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

    let dt = get_nowtime(time_zone);
    let fname = format!("user_list_{}.csv", dt.format("%Y%m%d"));
    let mut file = File::create(fname.clone())?;
    file.write_all(&wtr.into_inner()?)?;

    Ok(fname)
}

pub async fn export_user_list(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<HashMap<String, SystemSetting>>,
) -> Result<NamedFile, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data, &session).await?;
    let dbname = get_string_value(&session, "dbname")?;

    let system_setting = setting_map.get(&dbname);
    if system_setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let system_setting = system_setting.unwrap();

    let user = User::default();
    let users = match search_items(&db, &user).await {
        Ok(users) => users,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    match write_user_list(users, &system_setting.time_zone) {
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

fn write_book_list(books: Vec<Book>, time_zone: &str) -> Result<String, Box<dyn error::Error>> {
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

    let dt = get_nowtime(time_zone);
    let fname = format!("book_list_{}.csv", dt.format("%Y%m%d"));
    let mut file = File::create(fname.clone())?;
    file.write_all(&wtr.into_inner()?)?;

    Ok(fname)
}

pub async fn export_book_list(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<HashMap<String, SystemSetting>>,
) -> Result<NamedFile, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data, &session).await?;
    let dbname = get_string_value(&session, "dbname")?;
    let system_setting = setting_map.get(&dbname);
    if system_setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let system_setting = system_setting.unwrap();

    let book = Book::default();
    let books = match search_items(&db, &book).await {
        Ok(books) => books,
        Err(e) => {
            error!("{:?}", e);
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    match write_book_list(books, &system_setting.time_zone) {
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

fn write_transaction_list(
    items: Vec<TransactionItem>,
    time_zone: &str,
) -> Result<String, Box<dyn error::Error>> {
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

    let dt = get_nowtime(time_zone);
    let fname = format!("transaction_list_{}.csv", dt.format("%Y%m%d"));
    let mut file = File::create(fname.clone())?;
    file.write_all(&wtr.into_inner()?)?;

    Ok(fname)
}

pub async fn export_history_list(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<HashMap<String, SystemSetting>>,
) -> Result<NamedFile, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data, &session).await?;
    let dbname = get_string_value(&session, "dbname")?;
    let system_setting = setting_map.get(&dbname);
    if system_setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let system_setting = system_setting.unwrap();

    let item = TransactionItem::default();
    let transaction_items = Transaction::search(&db, &item).await;

    if transaction_items.len() == 0 {
        return Err(BibErrorResponse::DataNotFound(String::new()));
    }

    match write_transaction_list(transaction_items, &system_setting.time_zone) {
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
