use crate::views::path::Path;
use actix_web::web;
mod csv;

pub fn csv_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/csv"),
    };
    app.route(
        &base_path.define(String::from("/user")),
        web::get().to(csv::user),
    )
    .route(
        &base_path.define(String::from("/book")),
        web::get().to(csv::book),
    );
}
