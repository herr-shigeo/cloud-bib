use crate::views::path::Path;
use actix_web::web;
mod edit;

pub fn edit_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/edit"),
    };
    app.route(
        &base_path.define(String::from("/main")),
        web::get().to(edit::load),
    )
    .route(
        &base_path.define(String::from("/user")),
        web::post().to(edit::user),
    )
    .route(
        &base_path.define(String::from("/book")),
        web::post().to(edit::book),
    );
}
