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
) -> Result<(), BibErrorResponse> {
    if check_member_session(session).is_err() {
        info!("New member session");
        session
            .set("user_id", user_id)
            .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
    }
    Ok(())
}

pub fn check_session(session: &Session) -> Result<(), BibErrorResponse> {
    if let Some(dbname) = session
        .get::<String>("dbname")
        .map_err(|_| BibErrorResponse::NotAuthorized)?
    {
        debug!("SESSION: dbname = {}", dbname);
        Ok(())
    } else {
        Err(BibErrorResponse::NotAuthorized)
    }
}

pub fn check_member_session(session: &Session) -> Result<u32, BibErrorResponse> {
    if let Some(user_id) = session
        .get::<u32>("user_id")
        .map_err(|_| BibErrorResponse::NotAuthorized)?
    {
        debug!("SESSION user_id: {}", user_id);
        Ok(user_id)
    } else {
        Err(BibErrorResponse::NotAuthorized)
    }
}

pub fn check_any_session(session: &Session) -> Result<(), BibErrorResponse> {
    if let Ok(_) = check_member_session(session) {
        return Ok(());
    }
    check_session(session)
}
