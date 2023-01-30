use crate::error::BibErrorResponse;
use crate::item::{search_items, Book, User};
use crate::item::{SystemSetting, TransactionItem};
use crate::views::content_loader::read_file;
use crate::views::db_helper::get_db;
use crate::views::session::check_operator_session;
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

fn write_user_list(
    users: Vec<User>,
    prefix: &str,
    time_zone: &str,
) -> Result<String, Box<dyn error::Error>> {
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);

    wtr.write_record(&[
        "利用者ID",
        "氏名",
        "カナ",
        "利用者区分",
        "学年クラス",
        "備考",
        "登録日",
        "貸出回数",
        "",
    ])?;

    for user in users {
        wtr.serialize(User {
            id: user.id,
            name: user.name.clone(),
            kana: user.kana.clone(),
            category: user.category.clone(),
            grade: user.grade.clone(),
            remark: user.remark.clone(),
            register_date: user.register_date.clone(),
            borrowed_count: user.borrowed_count.clone(),
            reserved: user.reserved.clone(),
            borrowed_books: vec![],
        })?;
    }

    let dt = get_nowtime(time_zone);
    let fname = format!("user_list_{}_{}.csv", dt.format("%Y%m%d"), prefix);
    let mut file = File::create(fname.clone())?;
    file.write_all(&wtr.into_inner()?)?;

    Ok(fname)
}

pub async fn export_user_list(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<Mutex<HashMap<String, SystemSetting>>>,
) -> Result<NamedFile, BibErrorResponse> {
    let dbname = check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let setting_map = setting_map.lock().unwrap();
    let setting = setting_map.get(&dbname);
    if setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let setting = setting.unwrap().clone();
    drop(setting_map);

    let user = User::default();
    let users = match search_items(&db, &user).await {
        Ok(users) => users,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    match write_user_list(users, &dbname, &setting.time_zone) {
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

fn write_book_list(
    books: Vec<Book>,
    prefix: &str,
    time_zone: &str,
) -> Result<String, Box<dyn error::Error>> {
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);

    wtr.write_record(&[
        "図書ID",
        "タイトル",
        "保管場所",
        "図書分類",
        "破損状況",
        "著者",
        "出版社",
        "シリーズ",
        "巻数",
        "タイトルヨミ",
        "分類記号",
        "図書記号",
        "巻冊記号",
        "禁帯出",
        "備考",
        "登録日",
        "登録区分",
        "貸出回数",
        "",
        "",
    ])?;

    for book in books {
        wtr.serialize(Book {
            id: book.id,
            title: book.title.clone(),
            location: book.location.clone(),
            category: book.category.clone(),
            status: book.status.clone(),
            author: book.author.clone(),
            publisher: book.publisher.clone(),
            series: book.series.clone(),
            volume: book.volume.clone(),
            kana: book.kana.clone(),
            category_symbol: book.category_symbol.clone(),
            library_symbol: book.library_symbol.clone(),
            volume_symbol: book.volume_symbol.clone(),
            forbidden: book.forbidden.clone(),
            remark: book.remark.clone(),
            register_date: book.register_date.clone(),
            register_type: book.register_type.clone(),
            borrowed_count: book.borrowed_count.clone(),
            owner_id: None,
            return_deadline: None,
        })?;
    }

    let dt = get_nowtime(time_zone);
    let fname = format!("book_list_{}_{}.csv", dt.format("%Y%m%d"), prefix);
    let mut file = File::create(fname.clone())?;
    file.write_all(&wtr.into_inner()?)?;

    Ok(fname)
}

pub async fn export_book_list(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<Mutex<HashMap<String, SystemSetting>>>,
) -> Result<NamedFile, BibErrorResponse> {
    let dbname = check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let setting_map = setting_map.lock().unwrap();
    let setting = setting_map.get(&dbname);
    if setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let setting = setting.unwrap().clone();
    drop(setting_map);

    let book = Book::default();
    let books = match search_items(&db, &book).await {
        Ok(books) => books,
        Err(e) => {
            error!("{:?}", e);
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    match write_book_list(books, &dbname, &setting.time_zone) {
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
    prefix: &str,
    time_zone: &str,
) -> Result<String, Box<dyn error::Error>> {
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);

    wtr.write_record(&[
        "",
        "利用者ID",
        "利用者氏名",
        "図書ID",
        "図書タイトル",
        "貸出日",
        "返却日",
    ])?;

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
    let fname = format!("transaction_list_{}_{}.csv", dt.format("%Y%m%d"), prefix);
    let mut file = File::create(fname.clone())?;
    file.write_all(&wtr.into_inner()?)?;

    Ok(fname)
}

pub async fn export_history_list(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<Mutex<HashMap<String, SystemSetting>>>,
) -> Result<NamedFile, BibErrorResponse> {
    let dbname = check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let setting_map = setting_map.lock().unwrap();
    let setting = setting_map.get(&dbname);
    if setting.is_none() {
        return Err(BibErrorResponse::NotAuthorized);
    }
    let setting = setting.unwrap().clone();
    drop(setting_map);

    let item = TransactionItem::default();
    let transaction_items = Transaction::search(&db, &item).await;

    if transaction_items.len() == 0 {
        return Err(BibErrorResponse::DataNotFound(String::new()));
    }

    match write_transaction_list(transaction_items, &dbname, &setting.time_zone) {
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
