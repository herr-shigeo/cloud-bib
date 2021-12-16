use crate::views::content_loader::read_file;
use actix_web::web;
use actix_web::HttpResponse;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    pub user_id: String,
}

pub async fn load(form: web::Query<FormData>) -> HttpResponse {
    let user_id = &form.user_id;
    let mut html_data = read_file("src/html/user.html").unwrap();
    html_data = html_data.replace("{{USER_ID}}", user_id);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}
