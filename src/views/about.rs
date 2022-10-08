use crate::views::content_loader::read_file;
use actix_web::HttpResponse;

pub async fn load() -> HttpResponse {
    let html_data = read_file("src/html/about.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}
