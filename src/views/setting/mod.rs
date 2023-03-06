use crate::views::path::Path;
use actix_web::web;
mod setting;

pub fn setting_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/setting"),
    };
    app.route(
        &base_path.define(String::from("/main")),
        web::get().to(setting::load),
    )
    .route(
        &base_path.define(String::from("/get")),
        web::get().to(setting::get_setting),
    )
    .route(
        &base_path.define(String::from("/update/rental")),
        web::put().to(setting::update_rental_setting),
    )
    .route(
        &base_path.define(String::from("/update/barcode")),
        web::put().to(setting::update_barcode_setting),
    )
    .route(
        &base_path.define(String::from("/import_user")),
        web::post().to(setting::import_user_list),
    )
    .route(
        &base_path.define(String::from("/import_book")),
        web::post().to(setting::import_book_list),
    );
}
