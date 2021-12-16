use crate::views::path::Path;
use actix_web::web;
mod export;

pub fn export_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/export"),
    };
    app.route(
        &base_path.define(String::from("/main")),
        web::get().to(export::load),
    )
    .route(
        &base_path.define(String::from("/user")),
        web::get().to(export::export_user_list),
    )
    .route(
        &base_path.define(String::from("/book")),
        web::get().to(export::export_book_list),
    )
    .route(
        &base_path.define(String::from("/history")),
        web::get().to(export::export_history_list),
    );
}
