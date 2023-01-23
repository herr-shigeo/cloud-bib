use crate::error::BibErrorResponse;
use crate::item::atoi;
use crate::item::search_items;
use crate::item::User;
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::views::session::check_operator_session;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use log::debug;
use serde::Deserialize;
use shared_mongodb::ClientHolder;
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub id: String,
    pub name: String,
    pub kana: String,
    pub category: String,
}

pub async fn search_user(
    session: Session,
    form: web::Query<FormData>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    debug!("{:?}", form);
    check_operator_session(&session)?;

    let mut user = User::default();
    if form.id == "" {
        user.id = 0;
    } else {
        user.id = atoi(&form.id).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    }
    user.name = form.name.clone();
    user.kana = form.kana.clone();
    user.category = form.category.clone();
    get_user_list(data, &user, &session).await
}

async fn get_user_list(
    data: web::Data<Mutex<ClientHolder>>,
    user: &User,
    session: &Session,
) -> Result<HttpResponse, BibErrorResponse> {
    let db = get_db(&data, session).await?;

    let mut users = match search_items(&db, user).await {
        Ok(users) => users,
        Err(_) => {
            return Err(BibErrorResponse::UserNotFound(user.id));
        }
    };

    let mut reply = Reply::default();
    reply.user_list.append(&mut users);

    Ok(HttpResponse::Ok().json(reply))
}
