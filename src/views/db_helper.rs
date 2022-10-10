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
    session: Option<&Session>,
) -> Result<Database, BibErrorResponse> {
    let database_name: String = match session {
        Some(session) => {
            if let Some(dbname) = session
            .get::<String>("dbname")
            .map_err(|_| BibErrorResponse::NotAuthorized)?
            {
                dbname.clone()
            } else {
                return Err(BibErrorResponse::NotAuthorized);
            }
        }
        None => DB_SYSTEM_USERS.to_string(),
    };
    let db = database::get(data, &database_name)
        .await
        .map_err(|e| BibErrorResponse::DbConnectionError(e.to_string()))?;
    Ok(db)
}


