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

    let mut book = Book::default();
    book.id = 0;
    let books = match search_items(&db, &book).await {
        Ok(books) => books,
        Err(_) => {
            panic!("failed to search books");
        }
    };

    for mut book in books {
        let mut char_it = book.char.chars();
        let first_char = char_it.nth(0);
        let second_char = char_it.nth(0);
        if first_char.is_none() {
            continue;
        }
        if second_char.is_none() {
            continue;
        }
        //debug!("{}", book.char);
        //debug!("{}", first_char.unwrap() );
        //debug!("{}", second_char.unwrap() );
        if first_char.unwrap() == '緑' && second_char.unwrap() == '色' {
            let new_char: String = book
                .char
                .clone()
                .chars()
                .map(|x| match x {
                    '色' => '\0',
                    _ => x,
                })
                .collect();
            book.char = new_char;
            debug!("{:?}", book);
            match update_item(&db, &book).await {
                Ok(_) => {}
                Err(_) => {
                    panic!("update failed");
                }
            }
        }
    }

    info!("done");
}
