use actix_web::web;
mod about;
mod account;
mod auth;
pub mod cache;
mod constatns;
mod content_loader;
mod db_helper;
mod edit;
mod export;
mod history;
mod home;
mod index;
mod maintain;
mod manual;
mod member;
mod notation;
mod path;
mod privacy;
mod reply;
mod search;
mod session;
mod setting;
mod terms;
pub mod transaction;
mod user;
mod utils;
mod work;

use crate::views::path::Path;

pub fn views_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from(""),
    };
    app.route(
        &base_path.define(String::from("/")),
        web::get().to(about::load),
    )
    .route(
        &base_path.define(String::from("/css/{filename:.*}")),
        web::get().to(index::css_files),
    )
    .route(
        &base_path.define(String::from("/image/{filename:.*}")),
        web::get().to(index::image_files),
    )
    .route(
        &base_path.define(String::from("/js/{filename:.*}")),
        web::get().to(index::js_files),
    )
    .route(
        &base_path.define(String::from("/login")),
        web::get().to(index::load),
    )
    .route(
        &base_path.define(String::from("/notation")),
        web::get().to(notation::load),
    )
    .route(
        &base_path.define(String::from("/terms")),
        web::get().to(terms::load),
    )
    .route(
        &base_path.define(String::from("/privacy")),
        web::get().to(privacy::load),
    )
    .route(
        &base_path.define(String::from("/home")),
        web::get().to(home::load),
    )
    .route(
        &base_path.define(String::from("/user")),
        web::get().to(user::load),
    )
    .route(
        &base_path.define(String::from("/work")),
        web::post().to(work::process),
    )
    .route(
        &base_path.define(String::from("/history")),
        web::get().to(history::load),
    )
    .route(
        &base_path.define(String::from("/history/search")),
        web::get().to(history::search),
    );

    search::search_factory(app);
    setting::setting_factory(app);
    auth::auth_factory(app);
    export::export_factory(app);
    member::member_factory(app);
    edit::edit_factory(app);
    maintain::maintain_factory(app);
    account::account_factory(app);
    manual::manual_factory(app);
}
