use crate::error::*;
use crate::item::search_items;
use crate::item::BorrowedBook;
use crate::item::SystemSetting;
use crate::item::User;
use crate::views::content_loader::read_file;
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::views::session::check_session;
use crate::views::session::get_string_value;
use crate::views::utils::get_nowtime;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use chrono::NaiveDateTime;
use log::debug;
use serde::Serialize;
use shared_mongodb::ClientHolder;
use std::collections::HashMap;
use std::sync::Mutex;

pub async fn load(_session: Session) -> HttpResponse {
    let html_data = read_file("src/html/search.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

#[derive(Serialize, Debug)]
pub struct DelayedBook {
    pub user_id: u32,
    pub user_name: String,
    pub book: BorrowedBook,
}

pub async fn search_delayed_list(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<HashMap<String, SystemSetting>>,
) -> Result<HttpResponse, BibErrorResponse> {
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
        Err(_) => {
            return Err(BibErrorResponse::UserNotFound(user.id));
        }
    };

    let mut delayed_books: Vec<DelayedBook> = vec![];

    for user in users {
        for book in user.borrowed_books {
            let deadline = &book.return_deadline;
            let is_over = match check_deadline(deadline, &system_setting.time_zone) {
                Ok(is_over) => is_over,
                Err(e) => {
                    return Err(BibErrorResponse::SystemError(e.to_string()));
                }
            };
            debug!("deadline = {}", deadline);
            if is_over {
                let delayed_book = DelayedBook {
                    user_id: user.id,
                    user_name: user.name.clone(),
                    book: book,
                };
                delayed_books.push(delayed_book);
            }
            debug!("Is the deadline over? -> {}", is_over);
        }
    }

    if delayed_books.len() == 0 {
        return Err(BibErrorResponse::DataNotFound(String::new()));
    }

    let mut reply = Reply::default();
    reply.delayed_list = delayed_books;
    Ok(HttpResponse::Ok().json(reply))
}

fn check_deadline(deadline: &str, time_zone: &str) -> Result<bool, BibErrorResponse> {
    let deadline = match NaiveDateTime::parse_from_str(&deadline, "%Y/%m/%d %H:%M") {
        Ok(t) => t,
        Err(e) => {
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };

    let nowtime_string = format!("{}", get_nowtime(time_zone).format("%Y/%m/%d %H:%M"));
    let nowtime = match NaiveDateTime::parse_from_str(&nowtime_string, "%Y/%m/%d %H:%M") {
        Ok(t) => t,
        Err(e) => {
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };

    let is_over: bool = nowtime > deadline;
    debug!("{} > {} ? {}", nowtime, deadline, is_over);
    Ok(is_over)
}
