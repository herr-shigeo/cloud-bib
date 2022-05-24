use crate::error::*;
use crate::item::search_items;
use crate::item::BorrowedBook;
use crate::item::User;
use crate::views::content_loader::read_file;
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::views::session::check_session;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Berlin;
use log::debug;
use serde::Serialize;
use shared_mongodb::ClientHolder;
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
) -> Result<HttpResponse, BibErrorResponse> {
    check_session(&session)?;
    let db = get_db(&data).await?;

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
            let is_over = match check_deadline(deadline) {
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

fn check_deadline(deadline: &str) -> Result<bool, BibErrorResponse> {
    let deadline_berlin = match NaiveDateTime::parse_from_str(&deadline, "%Y/%m/%d %H:%M") {
        Ok(t) => t,
        Err(e) => {
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };

    let now_utc = Utc::now().naive_utc();
    let now_berlin = Berlin.from_utc_datetime(&now_utc);
    let now_string = format!("{}", now_berlin.format("%Y/%m/%d %H:%M"));
    let now_berlin = match NaiveDateTime::parse_from_str(&now_string, "%Y/%m/%d %H:%M") {
        Ok(t) => t,
        Err(e) => {
            return Err(BibErrorResponse::SystemError(e.to_string()));
        }
    };

    let is_over: bool = now_berlin > deadline_berlin;
    debug!("{} > {} ? {}", now_berlin, deadline_berlin, is_over);
    Ok(is_over)
}
