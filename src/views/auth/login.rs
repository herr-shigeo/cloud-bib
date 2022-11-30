use crate::error::BibErrorResponse;
use crate::item::{search_items, SystemUser};
use crate::views::db_helper::get_db_with_name;
use crate::views::reply::Reply;
use crate::views::session::*;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use shared_mongodb::ClientHolder;
use std::env;
use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
    static ref DB_COMMON_NAME: String =
        env::var("BIB_DB_NAME").expect("You must set the BIB_DB_NAME environment var!");
}

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
    let db = get_db_with_name(&data, &DB_COMMON_NAME.to_string()).await?;

    let mut system_user = SystemUser::default();
    system_user.uname = form.uname.clone();
    let mut system_user = match search_items(&db, &system_user).await {
        Ok(system_user) => system_user,
        Err(e) => {
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    if system_user.len() == 1 {
        loop {
            let system_user = system_user.pop().unwrap();

            let res = argon2::verify_encoded(&system_user.password, form.password.as_bytes())
                .map_err(|e| BibErrorResponse::SystemError(e.to_string()));
            if res.is_ok() {
                check_or_create_session(&session, &system_user.dbname)?;
                break;
            }

            if form.password.eq(&system_user.password) {
                check_or_create_session(&session, &system_user.dbname)?;
                break;
            }

            if system_user.uname == "demo" {
                check_or_create_session(&session, &system_user.dbname)?;
                break;
            }

            return Err(BibErrorResponse::LoginFailed);
        }
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}
