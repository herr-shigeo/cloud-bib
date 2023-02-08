use crate::views::path::Path;
use actix_web::web;
mod stripe;

pub fn stripe_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/stripe"),
    };
    app.route(
        &base_path.define(String::from("/webhook")),
        web::post().to(stripe::webhook),
    );
}
