use crate::views::path::Path;
use actix_web::web;
mod export;

pub fn export_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::new(),
    };
    app.route(
        &base_path.define(String::from("/export/main")),
        web::get().to(export::load),
    )
    .route(
        &base_path.define(String::from("/user/export")),
        web::get().to(export::export_user_list),
    )
    .route(
        &base_path.define(String::from("/book/export")),
        web::get().to(export::export_book_list),
    )
    .route(
        &base_path.define(String::from("/history/export")),
        web::get().to(export::export_history_list),
    );
}
