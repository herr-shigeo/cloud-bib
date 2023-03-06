use crate::item::{atoi, search_items_range, Book, User};
use crate::views::content_loader::read_file;
use crate::views::db_helper::get_db;
use crate::views::reply::Reply;
use crate::{error::BibErrorResponse, views::session::check_operator_session};
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use futures::TryFutureExt;
use serde::Deserialize;
use shared_mongodb::ClientHolder;
use std::sync::Mutex;

pub async fn load(_session: Session) -> HttpResponse {
    let html_data = read_file("src/html/barcode/index.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

#[derive(Deserialize, Debug)]
pub struct GenUserBarcodeGenPageForm {
    pub user_id_start: String,
    pub user_id_end: String,
    pub barcode_type: String,
    pub barcode_width_control: String,
    pub barcode_height_control: String,
    pub barcode_margin_control: String,
    pub barcode_size: String,
}

#[derive(Deserialize, Debug)]
pub struct GenUserBarcodeForm {
    pub user_id_start: String,
    pub user_id_end: String,
    pub barcode_size: String,
}

#[derive(Deserialize, Debug)]
pub struct GenBookBarcodeGenPageForm {
    pub book_id_start: String,
    pub book_id_end: String,
    pub barcode_type: String,
    pub barcode_width_control: String,
    pub barcode_height_control: String,
    pub barcode_margin_control: String,
    pub barcode_size: String,
}

#[derive(Deserialize, Debug)]
pub struct GenBookBarcodeForm {
    pub book_id_start: String,
    pub book_id_end: String,
    pub barcode_size: String,
}

pub async fn get_user_page(form: web::Query<GenUserBarcodeGenPageForm>) -> HttpResponse {
    let mut html_data = read_file("src/html/barcode/user.html").unwrap();
    html_data = html_data
        .replace("{{USER_ID_START}}", &form.user_id_start)
        .replace("{{USER_ID_END}}", &form.user_id_end)
        .replace("{{BARCODE_TYPE}}", &form.barcode_type)
        .replace("{{BARCODE_WIDTH}}", &form.barcode_width_control)
        .replace("{{BARCODE_HEIGHT}}", &form.barcode_height_control)
        .replace("{{BARCODE_MARGIN}}", &form.barcode_margin_control)
        .replace("{{BARCODE_SIZE}}", &form.barcode_size);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

pub async fn generate_user_barocde(
    session: Session,
    form: web::Json<GenUserBarcodeForm>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let user = User::default();
    let start_id =
        atoi(&form.user_id_start).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    let end_id =
        atoi(&form.user_id_end).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;

    let mut users = search_items_range(&db, &user, start_id, end_id)
        .map_err(|_| BibErrorResponse::BookNotFound(start_id))
        .await?;

    let barcode_size =
        atoi(&form.barcode_size).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;

    let mut reply = Reply::default();
    reply.user_list.append(&mut users);
    reply.barcode_size = barcode_size;

    Ok(HttpResponse::Ok().json(reply))
}

pub async fn get_book_page(form: web::Query<GenBookBarcodeGenPageForm>) -> HttpResponse {
    let mut html_data = read_file("src/html/barcode/book.html").unwrap();
    html_data = html_data
        .replace("{{BOOK_ID_START}}", &form.book_id_start)
        .replace("{{BOOK_ID_END}}", &form.book_id_end)
        .replace("{{BARCODE_TYPE}}", &form.barcode_type)
        .replace("{{BARCODE_WIDTH}}", &form.barcode_width_control)
        .replace("{{BARCODE_HEIGHT}}", &form.barcode_height_control)
        .replace("{{BARCODE_MARGIN}}", &form.barcode_margin_control)
        .replace("{{BARCODE_SIZE}}", &form.barcode_size);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

pub async fn generate_book_barcode(
    session: Session,
    form: web::Json<GenBookBarcodeForm>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_operator_session(&session)?;
    let db = get_db(&data, &session).await?;

    let book = Book::default();
    let start_id =
        atoi(&form.book_id_start).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;
    let end_id =
        atoi(&form.book_id_end).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;

    let mut books = search_items_range(&db, &book, start_id, end_id)
        .map_err(|_| BibErrorResponse::BookNotFound(start_id))
        .await?;

    let barcode_size =
        atoi(&form.barcode_size).map_err(|e| BibErrorResponse::InvalidArgument(e.to_string()))?;

    let mut reply = Reply::default();
    reply.book_list.append(&mut books);
    reply.barcode_size = barcode_size;

    Ok(HttpResponse::Ok().json(reply))
}
