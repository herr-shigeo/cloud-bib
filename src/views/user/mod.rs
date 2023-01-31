use crate::views::path::Path;
use actix_web::web;
mod user;

pub fn user_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/user"),
    };
    app.route(
        &base_path.define(String::from("/main")),
        web::get().to(user::load),
    );
}
