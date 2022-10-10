use crate::error::BibErrorResponse;
use actix_session::Session;
use actix_web::web;
use mongodb::Database;
use shared_mongodb::{database, ClientHolder};
use std::env;
use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
    static ref DB_NAME: String =
        env::var("BIB_DATABASE_NAME").expect("You must set the BIB_DATABSE_NAME environment var!");
    static ref DB_SYSTEM_USERS: String = "common".to_string();
}

pub async fn get_db(
    data: &web::Data<Mutex<ClientHolder>>,
    session: Option<&Session>,
) -> Result<Database, BibErrorResponse> {
    let database_name: &String = match session {
        Some(_session) => &DB_NAME,
        None => &DB_SYSTEM_USERS,
    };
    let db = database::get(data, database_name)
        .await
        .map_err(|e| BibErrorResponse::DbConnectionError(e.to_string()))?;
    Ok(db)
}
