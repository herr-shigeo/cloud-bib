use crate::views::path::Path;
use actix_web::web;
mod edit;

pub fn edit_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::new(),
    };
    app.route(
        &base_path.define(String::from("/edit/main")),
        web::get().to(edit::load),
    )
    .route(
        &base_path.define(String::from("/user/profile")),
        web::post().to(edit::insert_user),
    )
    .route(
        &base_path.define(String::from("/user/profile")),
        web::put().to(edit::update_user),
    )
    .route(
        &base_path.define(String::from("/user/profile")),
        web::delete().to(edit::delete_user),
    )
    .route(
        &base_path.define(String::from("/book/profile")),
        web::post().to(edit::insert_book),
    )
    .route(
        &base_path.define(String::from("/book/profile")),
        web::put().to(edit::update_book),
    )
    .route(
        &base_path.define(String::from("/book/profile")),
        web::delete().to(edit::delete_book),
    );
}
