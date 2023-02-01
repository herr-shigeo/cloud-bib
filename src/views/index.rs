use crate::views::content_loader::read_file;
use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Result};
use log::debug;
use std::path::PathBuf;

pub async fn load(req: HttpRequest) -> HttpResponse {
    let scheme = req
        .headers()
        .get("x-forwarded-proto")
        .map(|s| s.to_str().unwrap())
        .unwrap_or("");
    if scheme != "https" {
        let host = req
            .headers()
            .get("host")
            .map(|s| s.to_str().unwrap())
            .unwrap_or("");
        return HttpResponse::PermanentRedirect()
            .header("location", format!("https://{}{}", host, req.uri()))
            .finish();
    }

    let html_data = read_file("src/html/index.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

pub async fn css_files(req: HttpRequest) -> Result<NamedFile> {
    let file_name: String = req.match_info().query("filename").parse()?;
    let mut path = PathBuf::from("src/html/css");
    path.push(file_name);

    debug!("{:?}", path);
    Ok(NamedFile::open(path)?)
}

pub async fn image_files(req: HttpRequest) -> Result<NamedFile> {
    let file_name: String = req.match_info().query("filename").parse()?;
    let mut path = PathBuf::from("src/html/image");
    path.push(file_name);

    debug!("{:?}", path);
    Ok(NamedFile::open(path)?)
}

pub async fn js_files(req: HttpRequest) -> Result<NamedFile> {
    let file_name: String = req.match_info().query("filename").parse()?;
    let mut path = PathBuf::from("src/html/js");
    path.push(file_name);

    debug!("{:?}", path);
    Ok(NamedFile::open(path)?)
}
