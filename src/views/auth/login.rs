use crate::error::BibErrorResponse;
use crate::item::{search_items, SystemSetting};
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::views::session::*;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use shared_mongodb::ClientHolder;
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub uname: String,
    pub password: String,
}

pub async fn login(
    session: Session,
    form: web::Form<FormData>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    let db = get_db(&data).await?;

    let mut setting = SystemSetting::default();
    setting.id = 1;
    let mut setting = match search_items(&db, &setting).await {
        Ok(setting) => setting,
        Err(e) => {
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    let mut passed = false;
    if setting.len() == 1 {
        let setting = setting.pop().unwrap();
        if setting.uname == form.uname && setting.password == form.password {
            passed = true;
        }
    }
    if passed == false {
        return Err(BibErrorResponse::LoginFailed);
    }

    check_or_create_session(&session)?;

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}
