use crate::item::{Book, BorrowedBook, TransactionItem, User};
use crate::views::search::search::DelayedBook;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Reply {
    pub success: bool,
    pub path_to_home: String,
    pub transaction_list: Vec<TransactionItem>,
    pub user: User,
    pub borrowed_books: Vec<BorrowedBook>,
    pub user_list: Vec<User>,
    pub book_list: Vec<Book>,
    pub delayed_list: Vec<DelayedBook>,
    pub password: String,
    pub num_books: u32,
    pub num_days: u32,
    pub returned_book_title: String,
    pub returned_book_id: u32,
}

impl Reply {
    pub fn default() -> Self {
        Self {
            success: true,
            path_to_home: String::new(),
            transaction_list: vec![],
            borrowed_books: vec![],
            user: User::default(),
            user_list: vec![],
            book_list: vec![],
            delayed_list: vec![],
            password: String::new(),
            num_books: 0,
            num_days: 0,
            returned_book_title: String::new(),
            returned_book_id: 0,
        }
    }
}
