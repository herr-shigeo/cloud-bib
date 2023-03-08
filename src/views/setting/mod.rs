use crate::views::path::Path;
use actix_web::web;
mod setting;

pub fn setting_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::new(),
    };
    app.route(
        &base_path.define(String::from("/setting/main")),
        web::get().to(setting::load),
    )
    .route(
        &base_path.define(String::from("/setting/all")),
        web::get().to(setting::get_setting),
    )
    .route(
        &base_path.define(String::from("/setting/rental")),
        web::put().to(setting::update_rental_setting),
    )
    .route(
        &base_path.define(String::from("/setting/barcode")),
        web::put().to(setting::update_barcode_setting),
    )
    .route(
        &base_path.define(String::from("/user/profile/csv")),
        web::post().to(setting::import_user_list),
    )
    .route(
        &base_path.define(String::from("/book/profile/csv")),
        web::post().to(setting::import_book_list),
    );
}
