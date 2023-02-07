use actix_web::{web, HttpResponse};
mod account;
mod auth;
mod barcode;
pub mod cache;
mod constatns;
mod content_loader;
mod csv;
mod db_helper;
mod edit;
mod export;
mod history;
mod maintain;
mod manual;
mod member;
mod path;
mod reply;
mod search;
mod session;
mod setting;
pub mod transaction;
mod user;
mod utils;
mod work;
use std::{fs::File, io::Read, path::PathBuf};
pub mod reset_token;

#[cfg(not(local))]
fn redirect_to_https(req: &actix_web::HttpRequest) -> Option<HttpResponse> {
    let scheme = req
        .headers()
        .get("x-forwarded-proto")
        .map(|s| s.to_str().unwrap())
        .unwrap_or("");
    if scheme != "https" {
        let host = req
            .headers()
            .get("host")
            .map(|s| s.to_str().unwrap())
            .unwrap_or("");
        return Some(
            HttpResponse::PermanentRedirect()
                .header("location", format!("https://{}{}", host, req.uri()))
                .finish(),
        );
    }

    None
}

#[cfg(local)]
fn redirect_to_https(_req: &actix_web::HttpRequest) -> Option<HttpResponse> {
    None
}

async fn index_and_redirect_to_https(req: actix_web::HttpRequest) -> HttpResponse {
    match redirect_to_https(&req) {
        Some(res) => return res,
        None => {}
    }

    let mut file_name: String = req.match_info().query("filename").parse().unwrap();
    if file_name == "" || file_name.ends_with("/") {
        file_name += "index.html";
    }

    let mut path = PathBuf::from("src/html");
    path.push(file_name);

    let file = File::open(path);
    match file {
        Ok(mut f) => {
            let mut body = Vec::new();
            match f.read_to_end(&mut body) {
                Ok(_) => HttpResponse::Ok().body(body),
                Err(e) => {
                    HttpResponse::InternalServerError().body(format!("Error reading file: {:?}", e))
                }
            }
        }
        Err(e) => HttpResponse::NotFound().body(format!("File not found: {:?}", e)),
    }
}

pub fn views_factory(app: &mut web::ServiceConfig) {
    user::user_factory(app);
    work::work_factory(app);
    history::history_factory(app);
    search::search_factory(app);
    setting::setting_factory(app);
    auth::auth_factory(app);
    export::export_factory(app);
    member::member_factory(app);
    edit::edit_factory(app);
    maintain::maintain_factory(app);
    account::account_factory(app);
    manual::manual_factory(app);
    barcode::barcode_factory(app);
    csv::csv_factory(app);

    app.route("/{filename:.*}", web::get().to(index_and_redirect_to_https));
}
