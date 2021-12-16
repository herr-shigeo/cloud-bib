use crate::views::path::Path;
use actix_web::web;
mod book;
pub mod search;
mod user;

pub fn search_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/search"),
    };
    app.route(
        &base_path.define(String::from("/main")),
        web::get().to(search::load),
    )
    .route(
        &base_path.define(String::from("/delayed")),
        web::get().to(search::search_delayed_list),
    )
    .route(
        &base_path.define(String::from("/user")),
        web::get().to(user::search_user),
    )
    .route(
        &base_path.define(String::from("/book")),
        web::get().to(book::search_book),
    );
}
