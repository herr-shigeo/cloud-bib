use crate::views::path::Path;
use actix_web::web;
mod book;
pub mod search;
mod user;

pub fn search_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::new(),
    };
    app.route(
        &base_path.define(String::from("/search/main")),
        web::get().to(search::load),
    )
    .route(
        &base_path.define(String::from("/book/delayed/search")),
        web::get().to(search::search_delayed_list),
    )
    .route(
        &base_path.define(String::from("/user/search")),
        web::get().to(user::search_user),
    )
    .route(
        &base_path.define(String::from("/book/search")),
        web::get().to(book::search_book),
    )
    .route(
        &base_path.define(String::from("/book/isbn/search")),
        web::get().to(book::search_isbn),
    );
}
