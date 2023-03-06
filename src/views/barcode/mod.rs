use crate::views::path::Path;
use actix_web::web;
mod barcode;

pub fn barcode_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/barcode"),
    };
    app.route(
        &base_path.define(String::from("/main")),
        web::get().to(barcode::load),
    )
    .route(
        &base_path.define(String::from("/user/get_page")),
        web::get().to(barcode::get_user_page),
    )
    .route(
        &base_path.define(String::from("/user/generate")),
        web::post().to(barcode::generate_user_barocde),
    )
    .route(
        &base_path.define(String::from("/book/get_page")),
        web::get().to(barcode::get_book_page),
    )
    .route(
        &base_path.define(String::from("/book/generate")),
        web::post().to(barcode::generate_book_barcode),
    );
}
