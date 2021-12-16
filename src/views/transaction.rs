use crate::item::*;
use crate::item::{Book, User};
use crate::DbInstance;
use chrono::{TimeZone, Utc};
use chrono_tz::Europe::Berlin;
use log::{debug, info};
use std::error;
use std::sync::Mutex;

pub struct Transaction {
    pub max_counter: u32,
    pub counter: Mutex<u32>,
}

impl Transaction {
    pub fn new(max_counter: u32, counter: u32) -> Self {
        Transaction {
            max_counter: max_counter,
            counter: Mutex::new(counter),
        }
    }

    pub async fn search(db: &DbInstance, item: &TransactionItem) -> Vec<TransactionItem> {
        debug!("{:?}", item);
        let items = match search_items(db, item).await {
            Ok(items) => items,
            Err(e) => {
                info!("{:?}", e);
                vec![]
            }
        };
        items
    }

    pub async fn borrow(
        db: &DbInstance,
        counter: u32,
        user: &User,
        book: &Book,
    ) -> Result<(), Box<dyn error::Error>> {
        let utc = Utc::now().naive_utc();
        let dt = Berlin.from_utc_datetime(&utc);
        let item = TransactionItem {
            id: counter,
            user_id: user.id,
            user_name: user.name.clone(),
            book_id: book.id,
            book_title: book.title.clone(),
            borrowed_date: format!("{}", dt.format("%Y/%m/%d %H:%M")),
            returned_date: "".to_string(),
        };
        debug!("borrow: {:?}, counter={}", item, counter);
        update_item(db, &item).await
    }

    pub async fn unborrow(
        db: &DbInstance,
        counter: u32,
        user: &User,
        book: &Book,
        borrowed_date: String,
    ) -> Result<(), Box<dyn error::Error>> {
        let utc = Utc::now().naive_utc();
        let dt = Berlin.from_utc_datetime(&utc);
        let item = TransactionItem {
            id: counter,
            user_id: user.id,
            user_name: user.name.clone(),
            book_id: book.id,
            book_title: book.title.clone(),
            borrowed_date: borrowed_date,
            returned_date: format!("{}", dt.format("%Y/%m/%d %H:%M")),
        };
        debug!("unborrow: {:?}, counter={}", item, counter);
        update_item(db, &item).await
    }
}
