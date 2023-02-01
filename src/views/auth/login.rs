use crate::error::BibErrorResponse;
use crate::item::{search_items, SystemUser, User};
use crate::views::db_helper::get_db_with_name;
use crate::views::reply::Reply;
use crate::views::session::*;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use shared_mongodb::{database, ClientHolder};
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
                    .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
                if res {
                    create_session(
                        &session,
                        &system_user.uname,
                        &system_user.dbname,
                        &form.user_category,
                        None,
                    )?;
                    break;
                }
            } else if form.user_category == "operator" {
                // Verify the operator password
                let res = argon2::verify_encoded(
                    &system_user.operator_password,
                    form.password.as_bytes(),
                )
                .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
                if res {
                    create_session(
                        &session,
                        &system_user.uname,
                        &system_user.dbname,
                        &form.user_category,
                        None,
                    )?;
                    break;
                }
                if system_user.uname == "demo" {
                    create_session(
                        &session,
                        &system_user.uname,
                        &system_user.dbname,
                        &form.user_category,
                        None,
                    )?;
                    break;
                }
            } else if form.user_category == "user" {
                // Verify the user password
                let res =
                    argon2::verify_encoded(&system_user.user_password, form.password.as_bytes())
                        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
                if res {
                    // Verify the user name
                    let db = get_db_with_name(&data, &system_user.dbname).await?;
                    let user = User::new(&form.user_id, "", "", "", "", "", "")
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

                    create_session(
                        &session,
                        &system_user.uname,
                        &system_user.dbname,
                        &form.user_category,
                        Some(user.id),
                    )?;
                    break;
                }
            }
            return Err(BibErrorResponse::LoginFailed);
        }
    } else {
        return Err(BibErrorResponse::DataNotFound(form.uname.clone()));
    }

    let mut reply = Reply::default();

    match form.user_category.as_str() {
        "admin" => {
            reply.path_to_home = "/account/main".to_owned();
        }
        "operator" => {
            reply.path_to_home = "/home/".to_owned();
        }
        "user" => {
            reply.path_to_home = "/member/home".to_owned();
        }
        &_ => {}
    }
    Ok(HttpResponse::Ok().json(reply))
}
