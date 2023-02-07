use crate::error::BibErrorResponse;
use crate::views::content_loader::read_file;
use actix_files::NamedFile;
use actix_web::{web, HttpResponse};

pub async fn user() -> Result<NamedFile, BibErrorResponse> {
    let fname = "src/html/csv/user_list_example.csv";
    log::debug!("{:?}", fname);
    Ok(NamedFile::open(fname).map_err(|e| BibErrorResponse::SystemError(e.to_string()))?)
}

pub async fn book() -> Result<NamedFile, BibErrorResponse> {
    let fname = "src/html/csv/book_list_example.csv";
    log::debug!("{:?}", fname);
    Ok(NamedFile::open(fname).map_err(|e| BibErrorResponse::SystemError(e.to_string()))?)
}
