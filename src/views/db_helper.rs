use crate::error::BibErrorResponse;
use actix_web::web;
use mongodb::Database;
use shared_mongodb::{database, ClientHolder};
use std::env;
use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
    static ref DB_NAME: String =
        env::var("DATABASE_NAME").expect("You must set the DATABSE_NAME environment var!");
}

pub async fn get_db(data: &web::Data<Mutex<ClientHolder>>) -> Result<Database, BibErrorResponse> {
    let db = database::get(data, &DB_NAME)
        .await
        .map_err(|e| BibErrorResponse::DbConnectionError(e.to_string()))?;
    Ok(db)
}
