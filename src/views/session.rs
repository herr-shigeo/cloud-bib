use crate::error::BibErrorResponse;
use actix_session::Session;
use actix_web::Result;
use log::{debug, info};

pub fn create_session(
    session: &Session,
    uname: &String,
    dbname: &String,
    category: &String,
    user_id: Option<u32>,
) -> Result<(), BibErrorResponse> {
    if get_string_value(session, "dbname").is_ok() {
        info!("Override the session");
        session.remove("uname");
        session.remove("dbname");
        session.remove("category");
        session.remove("user_id");
    } else {
        info!("New session");
    }
    session
        .set("uname", uname)
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
    session
        .set("dbname", dbname)
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
    session
        .set("category", category)
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
    if user_id.is_some() {
        session
            .set("user_id", user_id)
            .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
    }
    Ok(())
}

pub fn check_admin_session(session: &Session) -> Result<String, BibErrorResponse> {
    check_session(session, "admin".to_owned())
}

pub fn check_operator_session(session: &Session) -> Result<String, BibErrorResponse> {
    check_session(session, "operator".to_owned())
}

pub fn check_user_session(session: &Session, user_id: u32) -> Result<String, BibErrorResponse> {
    let dbname = check_session(session, "user".to_owned())?;
    if get_user_id(session)? == user_id {
        return Ok(dbname);
    } else {
        return Err(BibErrorResponse::NotAuthorized);
    }
}

pub fn get_uname(session: &Session) -> Result<String, BibErrorResponse> {
    get_string_value(session, "uname")
}

pub fn get_user_id(session: &Session) -> Result<u32, BibErrorResponse> {
    get_int_value(session, "user_id")
}

fn check_session(session: &Session, category: String) -> Result<String, BibErrorResponse> {
    let c = get_string_value(session, "category")?;
    if c == category {
        return get_string_value(session, "dbname");
    } else {
        return Err(BibErrorResponse::NotAuthorized);
    }
}

fn get_string_value(session: &Session, key: &str) -> Result<String, BibErrorResponse> {
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

fn get_int_value(session: &Session, key: &str) -> Result<u32, BibErrorResponse> {
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
