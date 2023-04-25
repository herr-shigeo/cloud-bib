use crate::item::{BarcodeSetting, Book, BorrowedBook, RentalSetting, TransactionItem, User};
use crate::views::search::search::DelayedBook;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Reply {
    pub success: bool,
    pub redirect_to: String,
    pub transaction_list: Vec<TransactionItem>,
    pub user: User,
    pub borrowed_books: Vec<BorrowedBook>,
    pub user_list: Vec<User>,
    pub book_list: Vec<Book>,
    pub delayed_list: Vec<DelayedBook>,
    pub uname: String,
    pub email: String,
    pub plan: String,
    pub rental_setting: RentalSetting,
    pub barcode_setting: BarcodeSetting,
    pub returned_book_title: String,
    pub returned_book_id: u32,
    pub barcode_size: u32,
}

impl Default for Reply {
    fn default() -> Self {
        Self {
            success: true,
            redirect_to: String::new(),
            transaction_list: vec![],
            borrowed_books: vec![],
            user: User::default(),
            user_list: vec![],
            book_list: vec![],
            delayed_list: vec![],
            uname: String::new(),
            email: String::new(),
            plan: String::new(),
            rental_setting: RentalSetting::default(),
            barcode_setting: BarcodeSetting::default(),
            returned_book_title: String::new(),
            returned_book_id: 0,
            barcode_size: 0,
        }
    }
}
