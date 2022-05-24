use crate::item::create_unique_index;
use crate::item::TransactionItem;
use crate::views::cache::Cache;
use crate::views::transaction::*;
use actix_session::CookieSession;
use actix_web::{web, App, HttpServer};
use log::info;
use mongodb::options::{ClientOptions, Tls, TlsOptions};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use shared_mongodb::{database, ClientHolder};
use std::env;
use std::sync::Mutex;

mod error;
mod item;
mod views;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let port = env::var("PORT").unwrap_or("5000".to_string());
    let max_transaction_num = env::var("MAX_TRANSACTION_NUM").unwrap_or("100000".to_string());
    let max_transaction_num = max_transaction_num.parse::<u32>().unwrap();

    let client_uri =
        env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
    let mut client_options = match ClientOptions::parse(client_uri).await {
        Ok(client_options) => client_options,
        Err(e) => {
            panic!("{:?}", e);
        }
    };
    let tls_options = TlsOptions::builder().build();
    client_options.tls = Some(Tls::Enabled(tls_options));

    let client_holder = web::Data::new(Mutex::new(ClientHolder::new(client_options)));
    let db_name =
        env::var("DATABASE_NAME").expect("You must set the DATABSE_NAME environment var!");
    let db = database::get(&client_holder.clone(), &db_name)
        .await
        .unwrap();
    if let Err(e) = create_unique_index(&db).await {
        panic!("{:?}", e);
    }

    let mut last_counter = 0;
    let item = TransactionItem::default();
    let mut transaction_items = Transaction::search(&db, &item).await;
    if transaction_items.len() > 0 {
        let last_transaction = transaction_items.pop();
        last_counter = last_transaction.unwrap().id;
    }
    info!("last_counter = {}", last_counter);
    let transaction = web::Data::new(Transaction::new(max_transaction_num, last_counter));

    let cache = web::Data::new(Cache::new());
    cache.construct(&db).await;

    let mut csp_rng = ChaCha20Rng::from_entropy();
    let mut data = [0u8; 32];
    csp_rng.fill_bytes(&mut data);

    HttpServer::new(move || {
        let app = App::new()
            .configure(views::views_factory)
            .wrap(CookieSession::signed(&data).secure(true))
            .app_data(transaction.clone())
            .app_data(cache.clone())
            .app_data(client_holder.clone());
        return app;
    })
    .bind("0.0.0.0:".to_owned() + &port)?
    .run()
    .await
}
