use crate::views::path::Path;
use actix_web::web;
mod maintain;

pub fn maintain_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/maintain"),
    };
    app.route(
        &base_path.define(String::from("/main")),
        web::get().to(maintain::load),
    )
    .route(
        &base_path.define(String::from("/clear_status")),
        web::post().to(maintain::clear_status),
    )
    .route(
        &base_path.define(String::from("/clear_history")),
        web::post().to(maintain::clear_history),
    );
}
