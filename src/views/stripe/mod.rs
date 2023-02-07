use crate::views::path::Path;
use actix_web::web;
mod stripe;

pub fn stripe_factory(app: &mut web::ServiceConfig) {
    let base_path: Path = Path {
        prefix: String::from("/stripe"),
    };
    app.route(
        &base_path.define(String::from("/subscription/deleted")),
        web::post().to(stripe::subscription_deleted),
    )
    .route(
        &base_path.define(String::from("/subscription/updated")),
        web::post().to(stripe::subscription_updated),
    )
    .route(
        &base_path.define(String::from("/checkout/completed")),
        web::post().to(stripe::checkout_completed),
    );
}
