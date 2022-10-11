use crate::error::BibErrorResponse;
use actix_session::Session;
use actix_web::Result;
use log::{debug, info};

pub fn check_or_create_session(session: &Session, dbname: &String) -> Result<(), BibErrorResponse> {
    if check_session(session).is_err() {
        info!("New session");
        session
            .set("dbname", dbname)
            .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
    }
    Ok(())
}

pub fn check_or_create_member_session(
    session: &Session,
    user_id: u32,
    dbname: &String,
) -> Result<(), BibErrorResponse> {
    if check_member_session(session).is_err() {
        info!("New member session");
        session
            .set("user_id", user_id)
            .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
        session
            .set("dbname", dbname)
            .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
    }
    Ok(())
}

pub fn check_session(session: &Session) -> Result<(), BibErrorResponse> {
    get_string_value(session, "dbname")?;
    Ok(())
}

pub fn check_member_session(session: &Session) -> Result<u32, BibErrorResponse> {
    check_session(session)?;
    get_int_value(session, "user_id")
}

pub fn check_any_session(session: &Session) -> Result<(), BibErrorResponse> {
    if let Ok(_) = check_member_session(session) {
        return Ok(());
    }
    check_session(session)
}

pub fn get_string_value(session: &Session, key: &str) -> Result<String, BibErrorResponse> {
    if let Some(value) = session
        .get::<String>(key)
        .map_err(|_| BibErrorResponse::NotAuthorized)?
    {
        debug!("SESSION: key/value = {}/{}", key, value);
        Ok(value)
    } else {
        Err(BibErrorResponse::NotAuthorized)
    }
}

pub fn get_int_value(session: &Session, key: &str) -> Result<u32, BibErrorResponse> {
    if let Some(value) = session
        .get::<u32>(key)
        .map_err(|_| BibErrorResponse::NotAuthorized)?
    {
        debug!("SESSION: key/value = {}/{}", key, value);
        Ok(value)
    } else {
        Err(BibErrorResponse::NotAuthorized)
    }
}
