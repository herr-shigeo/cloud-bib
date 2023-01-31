use actix_web::web;
mod account;
use crate::views::path::Path;

pub fn account_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/account"),
    };
    app.route(
        &base_path.define(String::from("/register")),
        web::get().to(account::load_register),
    )
    .route(
        &base_path.define(String::from("/main")),
        web::get().to(account::load_main),
    )
    .route(
        &base_path.define(String::from("/get")),
        web::get().to(account::get),
    )
    .route(
        &base_path.define(String::from("/add")),
        web::post().to(account::add),
    )
    .route(
        &base_path.define(String::from("/update")),
        web::post().to(account::update),
    )
    .route(
        &base_path.define(String::from("/delete")),
        web::post().to(account::delete),
    )
    .route(
        &base_path.define(String::from("/reset")),
        web::post().to(account::reset),
    )
    .route(
        &base_path.define(String::from("/admin_password")),
        web::post().to(account::admin_password),
    )
    .route(
        &base_path.define(String::from("/operator_password")),
        web::post().to(account::operator_password),
    )
    .route(
        &base_path.define(String::from("/user_password")),
        web::post().to(account::user_password),
    );
}
