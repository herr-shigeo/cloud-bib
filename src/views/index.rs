use crate::views::content_loader::read_file;
use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Result};
use std::path::PathBuf;

pub async fn load() -> HttpResponse {
    let html_data = read_file("src/html/index.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

pub async fn css_files(req: HttpRequest) -> Result<NamedFile> {
    let file_name: String = req.match_info().query("filename").parse()?;
    let mut path = PathBuf::from("src/html/css");
    path.push(file_name);

    Ok(NamedFile::open(path)?)
}

pub async fn image_files(req: HttpRequest) -> Result<NamedFile> {
    let file_name: String = req.match_info().query("filename").parse()?;
    let mut path = PathBuf::from("src/html/image");
    path.push(file_name);

    Ok(NamedFile::open(path)?)
}

pub async fn js_files(req: HttpRequest) -> Result<NamedFile> {
    let file_name: String = req.match_info().query("filename").parse()?;
    let mut path = PathBuf::from("src/html/js");
    path.push(file_name);

    Ok(NamedFile::open(path)?)
}
