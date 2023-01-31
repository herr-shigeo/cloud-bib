use actix_web::web;
mod account;
mod auth;
pub mod cache;
mod constatns;
mod content_loader;
mod db_helper;
mod edit;
mod export;
mod history;
mod maintain;
mod manual;
mod member;
mod path;
mod reply;
mod search;
mod session;
mod setting;
pub mod transaction;
mod user;
mod utils;
mod work;
use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let mut file_name: String = req.match_info().query("filename").parse()?;
    if file_name == "" || file_name.ends_with("/") {
        file_name += "index.html";
    }

    let mut path = PathBuf::from("src/html");
    path.push(file_name);
    log::debug!("{:?}", path);

    Ok(NamedFile::open(path)?)
}

pub fn views_factory(app: &mut web::ServiceConfig) {
    user::user_factory(app);
    work::work_factory(app);
    history::history_factory(app);
    search::search_factory(app);
    setting::setting_factory(app);
    auth::auth_factory(app);
    export::export_factory(app);
    member::member_factory(app);
    edit::edit_factory(app);
    maintain::maintain_factory(app);
    account::account_factory(app);
    manual::manual_factory(app);

    app.route("/{filename:.*}", web::get().to(index));
}
