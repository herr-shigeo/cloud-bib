use crate::item::*;
use log::debug;
use mongodb::Database;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct BorrowCache {
    pub owner_id: u32,
    pub return_deadline: String,
}

impl BorrowCache {
    pub fn new(owner_id: u32, return_deadline: String) -> Self {
        Self {
            owner_id: owner_id,
            return_deadline: return_deadline.clone(),
        }
    }
}

pub struct Cache {
    pub borrowed_books: Mutex<HashMap<u32, BorrowCache>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            borrowed_books: Mutex::new(HashMap::new()),
        }
    }

    pub async fn construct(&self, db: &Database) {
        let mut user = User::default();
        user.id = 0;
        let users = match search_items(db, &user).await {
            Ok(users) => users,
            Err(_) => {
                panic!("failed to search users");
            }
        };

        for user in users {
            for book in user.borrowed_books {
                self.borrow(book.book_id, user.id, book.return_deadline);
            }
        }
        debug!("cache size = {}", self.borrowed_books.lock().unwrap().len());
    }

    pub fn get(&self, book_id: u32) -> Option<BorrowCache> {
        let borrowed_books = self.borrowed_books.lock().unwrap();
        let ret = borrowed_books.get(&book_id);
        if ret.is_none() {
            return None;
        }
        let info = ret.unwrap();
        let info = BorrowCache::new(info.owner_id, info.return_deadline.clone());
        drop(borrowed_books);
        Some(info)
    }

    pub fn borrow(
        &self,
        book_id: u32,
        user_id: u32,
        return_deadline: String,
    ) -> Option<BorrowCache> {
        debug!("book_id = {}, user_id = {}", book_id, user_id);
        let mut borrowed_books = self.borrowed_books.lock().unwrap();
        let ret = borrowed_books.insert(book_id, BorrowCache::new(user_id, return_deadline));
        drop(borrowed_books);
        debug!("ret = {:?}", ret);
        ret
    }

    pub fn unborrow(&self, book_id: u32) -> Option<BorrowCache> {
        debug!("book_id = {}", book_id);
        let mut borrowed_books = self.borrowed_books.lock().unwrap();
        let ret = borrowed_books.remove(&book_id);
        drop(borrowed_books);
        debug!("ret = {:?}", ret);
        ret
    }
}
