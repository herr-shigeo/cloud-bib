use crate::error::BibErrorResponse;
use actix_session::Session;
use actix_web::{HttpResponse, Result};

pub async fn logout(session: Session) -> Result<HttpResponse, BibErrorResponse> {
    session.purge();
    Err(BibErrorResponse::NotAuthorized)
}
