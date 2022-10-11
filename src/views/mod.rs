use actix_web::web;
mod about;
mod auth;
pub mod cache;
mod content_loader;
mod db_helper;
mod edit;
mod export;
mod history;
mod home;
mod index;
mod maintain;
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

use crate::views::path::Path;

pub fn views_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from(""),
    };
    app.route(
        &base_path.define(String::from("/")),
        web::get().to(index::load),
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
        &base_path.define(String::from("/about")),
        web::get().to(about::load),
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
    )
    .route(
        &base_path.define(String::from("/history/show_member")),
        web::get().to(history::show_member),
    );

    search::search_factory(app);
    setting::setting_factory(app);
    auth::auth_factory(app);
    export::export_factory(app);
    member::member_factory(app);
    edit::edit_factory(app);
    maintain::maintain_factory(app);
}
