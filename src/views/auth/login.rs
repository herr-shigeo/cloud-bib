use crate::error::BibErrorResponse;
use crate::item::{search_items, SystemSetting, SystemUser, User};
use crate::views::db_helper::get_db_with_name;
use crate::views::reply::Reply;
use crate::views::session::*;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use shared_mongodb::{database, ClientHolder};
use std::collections::HashMap;
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
    pub user_category: String,
    pub user_id: String,
}

pub async fn login(
    session: Session,
    form: web::Form<FormData>,
    data: web::Data<Mutex<ClientHolder>>,
    setting_map: web::Data<HashMap<String, SystemSetting>>,
) -> Result<HttpResponse, BibErrorResponse> {
    let db = get_db_with_name(&data, &DB_COMMON_NAME.to_string()).await?;

    // Verify the login ID
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

            if form.user_category == "admin" {
                // Verify the admin password
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
            } else if form.user_category == "user" {
                let setting = setting_map.get(&system_user.dbname);
                if setting.is_none() {
                    return Err(BibErrorResponse::NotAuthorized);
                }
                let setting = setting.unwrap();

                // Verify the user password
                let res =
                    argon2::verify_encoded(&setting.member_password, form.password.as_bytes())
                        .map_err(|e| BibErrorResponse::SystemError(e.to_string()));
                if res.is_err() {
                    if form.password != setting.member_password {
                        return Err(BibErrorResponse::LoginFailed);
                    }
                }

                // Verify the user name
                let db = get_db_with_name(&data, &system_user.dbname).await?;
                let user = User::new(&form.user_id, "", "", "", "", "")
                    .map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
                let mut users = match search_items(&db, &user).await {
                    Ok(users) => users,
                    Err(_) => {
                        database::disconnect(&data);
                        return Err(BibErrorResponse::UserNotFound(user.id));
                    }
                };
                if users.len() != 1 {
                    return Err(BibErrorResponse::DataDuplicated(0));
                }
                let user = users.pop().unwrap();

                check_or_create_member_session(&session, user.id, &system_user.dbname)?;
                break;
            } else {
                return Err(BibErrorResponse::LoginFailed);
            }
        }
    }

    let mut reply = Reply::default();
    if form.user_category == "admin" {
        reply.path_to_home = "/home".to_owned();
    } else {
        reply.path_to_home = "/member/home".to_owned();
    }

    Ok(HttpResponse::Ok().json(reply))
}
