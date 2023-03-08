use actix_web::web;
mod account;
use crate::views::path::Path;

pub fn account_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/account"),
    };
    app.route(
        &base_path.define(String::from("/main")),
        web::get().to(account::load),
    )
    .route(
        &base_path.define(String::from("/profile")),
        web::get().to(account::get),
    )
    .route(
        &base_path.define(String::from("/profile")),
        web::post().to(account::add),
    )
    .route(
        &base_path.define(String::from("/profile")),
        web::put().to(account::update),
    )
    .route(
        &base_path.define(String::from("/profile")),
        web::delete().to(account::delete),
    )
    .route(
        &base_path.define(String::from("/reset/request")),
        web::post().to(account::request_reset),
    )
    .route(
        &base_path.define(String::from("/reset/prepare")),
        web::get().to(account::prepare_reset),
    )
    .route(
        &base_path.define(String::from("/reset/do")),
        web::put().to(account::do_reset),
    )
    .route(
        &base_path.define(String::from("/admin/password")),
        web::put().to(account::admin_password),
    )
    .route(
        &base_path.define(String::from("/operator/password")),
        web::put().to(account::operator_password),
    )
    .route(
        &base_path.define(String::from("/user/password")),
        web::put().to(account::user_password),
    );
}
