use crate::item::create_unique_index;
use crate::item::search_items;
use crate::item::SystemSetting;
use crate::item::TransactionItem;
use crate::views::cache::Cache;
use crate::views::transaction::*;
use actix_session::CookieSession;
use actix_web::{web, App, HttpServer};
use item::SystemUser;
use log::info;
use mongodb::options::{ClientOptions, Tls, TlsOptions};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use shared_mongodb::{database, ClientHolder};
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;

mod error;
mod item;
mod views;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Get some environ vars
    let port = env::var("PORT").unwrap_or("5000".to_string());

    let client_uri =
        env::var("BIB_MONGODB_URI").expect("You must set the BIB_MONGODB_URI environment var!");
    let mut client_options = match ClientOptions::parse(client_uri).await {
        Ok(client_options) => client_options,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    // Set up the DB client holder
    let tls_options = TlsOptions::builder().build();
    client_options.tls = Some(Tls::Enabled(tls_options));
    let client_holder = web::Data::new(Mutex::new(ClientHolder::new(client_options)));

    // Get all users' names
    let db_name = env::var("BIB_DB_NAME").expect("You must set the BIB_DB_NAME environment var!");
    let db = database::get(&client_holder.clone(), &db_name)
        .await
        .unwrap();

    let system_user = SystemUser::default();
    let system_users = match search_items(&db, &system_user).await {
        Ok(system_users) => system_users,
        Err(_) => vec![],
    };
    let mut db_names_vec: Vec<String> = vec![];
    for system_user in system_users {
        db_names_vec.push(system_user.uname + "-system");
    }

    // Create a session data
    let mut csp_rng = ChaCha20Rng::from_entropy();
    let mut data = [0u8; 32];
    csp_rng.fill_bytes(&mut data);

    // Configure each DB
    let mut transaction_map: HashMap<String, Transaction> = HashMap::new();
    let mut cache_map: HashMap<String, Cache> = HashMap::new();
    let mut setting_map: HashMap<String, SystemSetting> = HashMap::new();

    for db_name in db_names_vec {
        let db = database::get(&client_holder.clone(), &db_name)
            .await
            .unwrap();

        if let Err(e) = create_unique_index(&db).await {
            panic!("{:?}", e);
        }

        //Read the SystemSetting
        let mut setting = SystemSetting::default();
        setting.id = 1;
        let setting = search_items(&db, &setting).await;
        let setting = setting.unwrap().pop().unwrap();
        let max_num_transactions = setting.max_num_transactions;
        setting_map.insert(db_name.to_string(), setting);

        // Create a Transaction
        let mut last_counter = 0;
        let item = TransactionItem::default();
        let mut transaction_items = Transaction::search(&db, &item).await;
        if transaction_items.len() > 0 {
            let last_transaction = transaction_items.pop();
            last_counter = last_transaction.unwrap().id;
        }
        info!(
            "last_counter/max_num_transactions = {}/{}",
            last_counter, max_num_transactions
        );
        let transaction = Transaction::new(max_num_transactions, last_counter);
        transaction_map.insert(db_name.to_string(), transaction);

        // Create a Cache
        let cache = Cache::new();
        cache.construct(&db).await;
        cache_map.insert(db_name.to_string(), cache);
    }
    let transaction_map = web::Data::new(Mutex::new(transaction_map));
    let cache_map = web::Data::new(Mutex::new(cache_map));
    let setting_map = web::Data::new(Mutex::new(setting_map));

    HttpServer::new(move || {
        let app = App::new()
            .configure(views::views_factory)
            .wrap(CookieSession::signed(&data).secure(true))
            .app_data(transaction_map.clone())
            .app_data(cache_map.clone())
            .app_data(setting_map.clone())
            .app_data(client_holder.clone());
        return app;
    })
    .bind("0.0.0.0:".to_owned() + &port)?
    .run()
    .await
}
