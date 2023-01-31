use crate::views::path::Path;
use actix_web::web;
mod work;

pub fn work_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/work"),
    };
    app.route(
        &base_path.define(String::from("/process")),
        web::post().to(work::process),
    );
}
