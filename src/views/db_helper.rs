use crate::error::BibErrorResponse;
use actix_session::Session;
use actix_web::web;
use mongodb::Database;
use shared_mongodb::{database, ClientHolder};
use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
    static ref DB_SYSTEM_USERS: String = "common".to_string();
}

pub async fn get_db(
    data: &web::Data<Mutex<ClientHolder>>,
    session: &Session,
) -> Result<Database, BibErrorResponse> {
    if let Some(dbname) = session
        .get::<String>("dbname")
        .map_err(|_| BibErrorResponse::NotAuthorized)?
    {
        let db = database::get(data, &dbname)
            .await
            .map_err(|e| BibErrorResponse::DbConnectionError(e.to_string()))?;
        return Ok(db);
    }
    return Err(BibErrorResponse::NotAuthorized);
}

pub async fn get_db_with_name(
    data: &web::Data<Mutex<ClientHolder>>,
    dbname: &String,
) -> Result<Database, BibErrorResponse> {
    let db = database::get(data, dbname)
        .await
        .map_err(|e| BibErrorResponse::DbConnectionError(e.to_string()))?;
    Ok(db)
}
