use crate::views::path::Path;
use actix_web::web;
mod member;

pub fn member_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/member"),
    };
    app.route(
        &base_path.define(String::from("")),
        web::get().to(member::load),
    )
    .route(
        &base_path.define(String::from("/login")),
        web::post().to(member::login),
    )
    .route(
        &base_path.define(String::from("/news")),
        web::get().to(member::load_news),
    )
    .route(
        &base_path.define(String::from("/borrowed_books")),
        web::get().to(member::borrowed_books),
    )
    .route(
        &base_path.define(String::from("/search/main")),
        web::get().to(member::load_search),
    )
    .route(
        &base_path.define(String::from("/home")),
        web::get().to(member::load_home),
    );
}
