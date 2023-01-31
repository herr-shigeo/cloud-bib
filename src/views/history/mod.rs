use crate::views::path::Path;
use actix_web::web;
mod history;

pub fn history_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/history"),
    };
    app.route(
        &base_path.define(String::from("/main")),
        web::get().to(history::load),
    )
    .route(
        &base_path.define(String::from("/search")),
        web::get().to(history::search),
    );
}
