use crate::db_client::*;
use crate::item::TransactionItem;
use crate::views::transaction::*;
use actix_session::CookieSession;
use actix_web::{web, App, HttpServer};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::env;
use std::sync::Mutex;

mod db_client;
mod error;
mod item;
mod views;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let port = env::var("PORT").unwrap_or("5000".to_string());
    let max_transaction_num = env::var("MAX_TRANSACTION_NUM").unwrap_or("2000000".to_string());
    let max_transaction_num = max_transaction_num.parse::<u32>().unwrap();

    let db_client = web::Data::new(Mutex::new(DbClient::new()));
    let db = get_db(&db_client.clone()).await.unwrap();

    let item = TransactionItem::default();
    let transaction_items = Transaction::search(&db, &item);
    let transaction_counter: u32 = transaction_items.await.len().try_into().unwrap();
    let transaction_counter: u32 = (transaction_counter % max_transaction_num)
        .try_into()
        .unwrap() + 1;
    let transaction = web::Data::new(Transaction::new(max_transaction_num, transaction_counter));

    let mut csp_rng = ChaCha20Rng::from_entropy();
    let mut data = [0u8; 32];
    csp_rng.fill_bytes(&mut data);

    HttpServer::new(move || {
        let app = App::new()
            .configure(views::views_factory)
            .wrap(CookieSession::signed(&data).secure(true))
            .app_data(transaction.clone())
            .app_data(db_client.clone());
        return app;
    })
    .bind("0.0.0.0:".to_owned() + &port)?
    .run()
    .await
}
