use crate::views::path::Path;
use actix_files::Files;
use actix_web::web;

use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let mut file_name: String = req.match_info().query("filename").parse()?;
    if file_name == "" || file_name.ends_with("/") {
        file_name += "index.html";
    }

    let mut path = PathBuf::from("src/html/manual");
    path.push(file_name);

    Ok(NamedFile::open(path)?)
}

pub fn manual_factory(app: &mut web::ServiceConfig) {
    app.route("/manual/{filename:.*}", web::get().to(index));
}
