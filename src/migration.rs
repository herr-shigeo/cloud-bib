use crate::item::create_unique_index;
use crate::item::TransactionItem;
use crate::item::*;
use crate::views::cache::Cache;
use crate::views::transaction::*;
use actix_session::CookieSession;
use actix_web::{web, App, HttpServer};
use log::{debug, info};
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
async fn main() {
    env_logger::init();

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

    let mut user = User::default();
    user.id = 0;
    let users = match search_items(&db, &user).await {
        Ok(users) => users,
        Err(_) => {
            panic!("failed to search users");
        }
    };

    for user in users {
        let mut user2 = match UserV2::new(
            "00000",
            &user.name,
            &user.kana,
            &user.category,
            &user.remark,
            &user.register_date,
        ) {
            Ok(user2) => user2,
            Err(_) => {
                panic!("failed to get users");
            }
        };
        user2.id = user.id;
        for book in user.borrowed_books {
            let mut book2 = BorrowedBookV2::default();
            book2.book_id = book.book_id;
            book2.book_title = book.book_title;
            book2.borrowed_date = book.borrowed_date;
            book2.return_deadline = book.return_deadline;
            book2.transaction_id = book.transaction_id;
            book2.char = "キャラ不明".to_string();
            user2.borrowed_books.push(book2);
        }
        debug!("{:?}", user2);
        match update_item(&db, &user2).await {
            Ok(_) => {}
            Err(_) => {
                panic!("update failed");
            }
        }
    }

    info!("done");
}
