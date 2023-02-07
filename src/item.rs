use async_trait::async_trait;
use bson::Document;
use chrono::{DateTime, Duration};
use chrono_tz::Tz;
use futures::stream::TryStreamExt;
use log::info;
use mongodb::bson::doc;
use mongodb::options::*;
use mongodb::Database;
use mongodb::{Collection, IndexModel};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error;
use std::io::{Error, ErrorKind};

const NUM_SEARCH_ITEMS_MAX: i64 = 100000;

#[async_trait]
pub trait Entity {
    async fn insert(&self, db: &Database) -> Result<(), Box<dyn error::Error>>;
    async fn update(&self, db: &Database) -> Result<(), Box<dyn error::Error>>;
    async fn delete(&self, db: &Database) -> Result<(), Box<dyn error::Error>>;
    async fn delete_all(&self, db: &Database) -> Result<(), Box<dyn error::Error>>;

    async fn search(&self, db: &Database) -> Result<Vec<Self>, Box<dyn error::Error>>
    where
        Self: std::marker::Sized;

    async fn search_range(
        &self,
        db: &Database,
        start_id: u32,
        end_id: u32,
    ) -> Result<Vec<Self>, Box<dyn error::Error>>
    where
        Self: std::marker::Sized;

    fn get_collection_name(&self) -> &str;

    fn get_collection(&self, db: &Database) -> Collection<Self>
    where
        Self: std::marker::Sized,
    {
        db.collection::<Self>(self.get_collection_name())
    }

    async fn create_unique_index(
        &self,
        db: &Database,
        field: &str,
    ) -> Result<(), Box<dyn error::Error>>
    where
        Self: std::marker::Sized,
        Self: std::marker::Send,
    {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! {field: 1})
            .options(options)
            .build();
        let collection = self.get_collection(db);
        collection.create_index(model, None).await?;
        Ok(())
    }
}

pub async fn insert_item<T: Entity>(db: &Database, item: &T) -> Result<(), Box<dyn error::Error>> {
    item.insert(db).await
}

pub async fn update_item<T: Entity>(db: &Database, item: &T) -> Result<(), Box<dyn error::Error>> {
    item.update(db).await
}

pub async fn delete_item<T: Entity>(db: &Database, item: &T) -> Result<(), Box<dyn error::Error>> {
    item.delete(db).await
}

pub async fn delete_item_all<T: Entity>(
    db: &Database,
    item: &T,
) -> Result<(), Box<dyn error::Error>> {
    item.delete_all(db).await
}

pub async fn search_items<T: Entity>(
    db: &Database,
    item: &T,
) -> Result<Vec<T>, Box<dyn error::Error>> {
    item.search(db).await
}

pub async fn search_items_range<T: Entity>(
    db: &Database,
    item: &T,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<T>, Box<dyn error::Error>> {
    item.search_range(db, start_id, end_id).await
}

pub async fn search_item<T: Entity>(db: &Database, item: &T) -> Result<T, Box<dyn error::Error>> {
    let mut items = item.search(db).await?;
    if items.len() == 1 {
        Ok(items.pop().unwrap())
    } else {
        Err(Box::new(Error::new(
            ErrorKind::Other,
            "Multiple items are found".to_string(),
        )))
    }
}

pub async fn create_unique_index(db: &Database) -> Result<(), Box<dyn error::Error>> {
    let item = User::default();
    item.create_unique_index(db, "id").await?;
    let item = Book::default();
    item.create_unique_index(db, "id").await?;
    Ok(())
}

pub fn atoi(a: &str) -> Result<u32, Box<dyn error::Error>> {
    let i: u32 = a.to_string().parse()?;
    Ok(i)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub kana: String,
    pub category: String,
    pub grade: String,
    pub remark: String,
    pub register_date: String,
    pub borrowed_count: u32,
    pub reserved: String,
    pub borrowed_books: Vec<BorrowedBook>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MonthlyPlan {
    Free,
    Light,
    Standard,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemUser {
    pub uname: String,
    pub email: String,
    pub password: String,
    pub operator_password: String,
    pub user_password: String,
    pub dbname: String,
    pub plan: MonthlyPlan,
    pub subscription_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BorrowedBook {
    pub book_id: u32,
    pub book_title: String,
    pub borrowed_date: String,
    pub return_deadline: String,
    pub transaction_id: u32,
    pub location: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Book {
    pub id: u32,
    pub title: String,
    pub location: String,
    pub category: String,
    pub status: String,
    pub author: String,
    pub publisher: String,
    pub published_date: String,
    pub series: String,
    pub page: String,
    pub volume: String,
    pub kana: String,
    pub category_symbol: String,
    pub library_symbol: String,
    pub volume_symbol: String,
    pub forbidden: String,
    pub remark: String,
    pub register_type: String,
    pub register_date: String,

    pub borrowed_count: u32,
    pub owner_id: Option<u32>,
    pub return_deadline: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RentalSetting {
    pub id: u32,
    pub num_books: u32,
    pub num_days: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemSetting {
    pub id: u32,
    pub max_num_transactions: u32,
    pub max_registered_users: u32,
    pub max_registered_books: u32,
    pub time_zone: String,
    pub num_threads: u32,
    pub max_parallel_registrations: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionItem {
    pub id: u32,
    pub user_id: u32,
    pub user_name: String,
    pub book_id: u32,
    pub book_title: String,
    pub borrowed_date: String,
    pub returned_date: String,
}

impl User {
    pub fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            kana: String::new(),
            category: String::new(),
            grade: String::new(),
            remark: String::new(),
            register_date: String::new(),
            borrowed_count: 0,
            reserved: String::new(),
            borrowed_books: vec![],
        }
    }

    pub fn new(
        id: &str,
        name: &str,
        kana: &str,
        category: &str,
        grade: &str,
        remark: &str,
        register_date: &str,
    ) -> Result<Self, Box<dyn error::Error>> {
        let r = Self {
            id: atoi(id)?,
            name: name.to_string(),
            kana: kana.to_string(),
            category: category.to_string(),
            grade: grade.to_string(),
            remark: remark.to_string(),
            register_date: register_date.to_string(),
            borrowed_count: 0,
            reserved: String::new(),
            borrowed_books: vec![],
        };
        Ok(r)
    }
}

impl SystemUser {
    pub fn default() -> Self {
        Self {
            uname: String::new(),
            email: String::new(),
            password: String::new(),
            operator_password: String::new(),
            user_password: String::new(),
            dbname: String::new(),
            plan: MonthlyPlan::Free,
            subscription_id: String::new(),
        }
    }
}

#[allow(dead_code)]
impl BorrowedBook {
    pub fn default() -> Self {
        Self {
            book_id: 0,
            book_title: String::new(),
            borrowed_date: String::new(),
            return_deadline: String::new(),
            transaction_id: 0,
            location: String::new(),
        }
    }

    pub fn new(
        id: u32,
        title: &str,
        nowtime: DateTime<Tz>,
        borrowing_days: i64,
        transaction_id: u32,
        location: String,
    ) -> Self {
        let deadline = nowtime + Duration::days(borrowing_days);
        Self {
            book_id: id,
            book_title: title.to_string(),
            borrowed_date: format!("{}", nowtime.format("%Y/%m/%d %H:%M")),
            return_deadline: format!("{}", deadline.format("%Y/%m/%d %H:%M")),
            transaction_id: transaction_id,
            location: location,
        }
    }
}

impl Book {
    pub fn default() -> Self {
        Self {
            id: 0,
            title: String::new(),
            location: String::new(),
            category: String::new(),
            status: String::new(),
            author: String::new(),
            publisher: String::new(),
            published_date: String::new(),
            series: String::new(),
            page: String::new(),
            volume: String::new(),
            kana: String::new(),
            category_symbol: String::new(),
            library_symbol: String::new(),
            volume_symbol: String::new(),
            forbidden: String::new(),
            remark: String::new(),
            register_type: String::new(),
            register_date: String::new(),

            borrowed_count: 0,
            owner_id: None,
            return_deadline: None,
        }
    }

    pub fn new(
        id: &str,
        title: &str,
        location: &str,
        category: &str,
        status: &str,
        author: &str,
        publisher: &str,
        published_date: &str,
        series: &str,
        volume: &str,
        page: &str,
        kana: &str,
        category_symbol: &str,
        library_symbol: &str,
        volume_symbol: &str,
        forbidden: &str,
        remark: &str,
        register_date: &str,
        register_type: &str,
    ) -> Result<Self, Box<dyn error::Error>> {
        let r = Self {
            id: atoi(id)?,
            title: title.to_string(),
            location: location.to_string(),
            category: category.to_string(),
            status: status.to_string(),
            author: author.to_string(),
            publisher: publisher.to_string(),
            published_date: published_date.to_string(),
            series: series.to_string(),
            volume: volume.to_string(),
            page: page.to_string(),
            kana: kana.to_string(),
            category_symbol: category_symbol.to_string(),
            library_symbol: library_symbol.to_string(),
            volume_symbol: volume_symbol.to_string(),
            forbidden: forbidden.to_string(),
            remark: remark.to_string(),
            register_type: register_type.to_string(),
            register_date: register_date.to_string(),
            borrowed_count: 0,
            owner_id: None,
            return_deadline: None,
        };
        Ok(r)
    }
}

impl RentalSetting {
    pub fn default() -> Self {
        Self {
            id: 0,
            num_books: 10,
            num_days: 14,
        }
    }

    pub fn new(num_books: &str, num_days: &str) -> Result<Self, Box<dyn error::Error>> {
        let r = Self {
            id: 0,
            num_books: atoi(num_books)?,
            num_days: atoi(num_days)?,
        };
        Ok(r)
    }
}

impl SystemSetting {
    pub fn default() -> Self {
        Self {
            id: 0,
            max_num_transactions: 0,
            max_registered_users: 0,
            max_registered_books: 0,
            time_zone: String::from("Tokyo"),
            num_threads: 10,
            max_parallel_registrations: 1000,
        }
    }
}

impl TransactionItem {
    pub fn default() -> Self {
        Self {
            id: 0,
            user_id: 0,
            user_name: String::new(),
            book_id: 0,
            book_title: String::new(),
            borrowed_date: String::new(),
            returned_date: String::new(),
        }
    }
    pub fn new(user_id: u32, user_name: &str, book_id: u32, book_title: &str) -> Self {
        Self {
            id: 0,
            user_id: user_id,
            user_name: user_name.to_string(),
            book_id: book_id,
            book_title: book_title.to_string(),
            borrowed_date: String::new(),
            returned_date: String::new(),
        }
    }
}

#[async_trait]
impl Entity for User {
    async fn insert(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let collection = self.get_collection(db);
        collection.insert_one(self, None).await?;
        Ok(())
    }

    async fn update(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let query = doc! { "id" : self.id };
        let update = bson::to_bson(self).unwrap();
        let update = doc! { "$set" : update };
        let collection = self.get_collection(db);
        collection.update(query, update, true).await
    }

    async fn delete(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let query = doc! { "id" : self.id };
        let collection = self.get_collection(db);
        collection.delete(query).await
    }

    async fn delete_all(&self, _db: &Database) -> Result<(), Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    async fn search(&self, db: &Database) -> Result<Vec<Self>, Box<dyn error::Error>> {
        let mut query = doc! { "id": { "$gt": 0 }};

        if self.id != 0 {
            query = doc! { "id": self.id };
        } else if self.name != "" {
            query = doc! { "name": {"$regex": &self.name} };
        } else if self.kana != "" {
            query = doc! { "kana": {"$regex": &self.kana} };
        } else if self.category != "" {
            query = doc! { "category": {"$regex": &self.category} };
        }

        let collection = self.get_collection(db);
        collection.search(query).await
    }

    async fn search_range(
        &self,
        db: &Database,
        start_id: u32,
        end_id: u32,
    ) -> Result<Vec<Self>, Box<dyn error::Error>> {
        let query = doc! { "$and": [ {"id": { "$gte": start_id }}, {"id": { "$lte": end_id }} ]};

        let collection = self.get_collection(db);
        collection.search(query).await
    }

    fn get_collection_name(&self) -> &str {
        "users2"
    }
}

#[async_trait]
impl Entity for SystemUser {
    async fn insert(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let collection = self.get_collection(db);
        collection.insert_one(self, None).await?;
        Ok(())
    }

    async fn update(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let query = doc! { "uname" : &self.uname };
        let update = bson::to_bson(self).unwrap();
        let update = doc! { "$set" : update };
        let collection = self.get_collection(db);
        collection.update(query, update, false).await
    }

    async fn delete(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let query = doc! { "uname" : &self.uname };
        let collection = self.get_collection(db);
        collection.delete(query).await
    }

    async fn delete_all(&self, _db: &Database) -> Result<(), Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    async fn search(&self, db: &Database) -> Result<Vec<Self>, Box<dyn error::Error>> {
        let mut query = doc! {};
        if self.uname != "" {
            query = doc! { "uname": &self.uname};
        }
        let collection = self.get_collection(db);
        collection.search(query).await
    }

    async fn search_range(
        &self,
        _db: &Database,
        _start_id: u32,
        _end_id: u32,
    ) -> Result<Vec<Self>, Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    fn get_collection_name(&self) -> &str {
        "system-users"
    }
}

#[async_trait]
impl Entity for Book {
    async fn insert(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let collection = self.get_collection(db);
        collection.insert_one(self, None).await?;
        Ok(())
    }

    async fn update(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let query = doc! { "id" : self.id };
        let update = bson::to_bson(self).unwrap();
        let update = doc! { "$set" : update };
        let collection = self.get_collection(db);
        collection.update(query, update, true).await
    }

    async fn delete(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let query = doc! { "id" : self.id };
        let collection = self.get_collection(db);
        collection.delete(query).await
    }

    async fn delete_all(&self, _db: &Database) -> Result<(), Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    async fn search(&self, db: &Database) -> Result<Vec<Self>, Box<dyn error::Error>> {
        let mut query = doc! { "id": { "$gt": 0 }};

        if self.id != 0 {
            query = doc! { "id": self.id };
        } else if self.title != "" {
            query = doc! { "title": {"$regex": &self.title} };
        } else if self.kana != "" {
            query = doc! { "kana": {"$regex": &self.kana} };
        } else if self.author != "" {
            query = doc! { "author": {"$regex": &self.author} };
        }

        let collection = self.get_collection(db);
        collection.search(query).await
    }

    async fn search_range(
        &self,
        db: &Database,
        start_id: u32,
        end_id: u32,
    ) -> Result<Vec<Self>, Box<dyn error::Error>> {
        let query = doc! { "$and": [ {"id": { "$gte": start_id }}, {"id": { "$lte": end_id }} ]};

        let collection = self.get_collection(db);
        collection.search(query).await
    }

    fn get_collection_name(&self) -> &str {
        "books"
    }
}

#[async_trait]
impl Entity for RentalSetting {
    async fn insert(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let collection = self.get_collection(db);
        collection.insert_one(self, None).await?;
        Ok(())
    }

    async fn update(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let query = doc! { "id" : self.id };
        let update = bson::to_bson(self).unwrap();
        let update = doc! { "$set" : update };
        let collection = self.get_collection(db);
        collection.update(query, update, false).await
    }

    async fn delete(&self, _db: &Database) -> Result<(), Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    async fn delete_all(&self, _db: &Database) -> Result<(), Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    async fn search(&self, db: &Database) -> Result<Vec<Self>, Box<dyn error::Error>> {
        let query = doc! { "$or" : [{"id": self.id}] };
        let collection = self.get_collection(db);
        collection.search(query).await
    }

    async fn search_range(
        &self,
        _db: &Database,
        _start_id: u32,
        _end_id: u32,
    ) -> Result<Vec<Self>, Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    fn get_collection_name(&self) -> &str {
        "rental-setting"
    }
}

#[async_trait]
impl Entity for SystemSetting {
    async fn insert(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let collection = self.get_collection(db);
        collection.insert_one(self, None).await?;
        Ok(())
    }

    async fn update(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let query = doc! { "id" : self.id };
        let update = bson::to_bson(self).unwrap();
        let update = doc! { "$set" : update };
        let collection = self.get_collection(db);
        collection.update(query, update, false).await
    }

    async fn delete(&self, _db: &Database) -> Result<(), Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    async fn delete_all(&self, _db: &Database) -> Result<(), Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    async fn search(&self, db: &Database) -> Result<Vec<Self>, Box<dyn error::Error>> {
        let query = doc! { "$or" : [{"id": self.id}] };
        let collection = self.get_collection(db);
        collection.search(query).await
    }

    async fn search_range(
        &self,
        _db: &Database,
        _start_id: u32,
        _end_id: u32,
    ) -> Result<Vec<Self>, Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    fn get_collection_name(&self) -> &str {
        "system-setting"
    }
}

#[async_trait]
impl Entity for TransactionItem {
    async fn insert(&self, _db: &Database) -> Result<(), Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    async fn update(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let query = doc! { "id": self.id };
        let update = bson::to_bson(self).unwrap();
        let update = doc! { "$set" : update };
        let collection = self.get_collection(db);
        collection.update(query, update, true).await
    }

    async fn delete(&self, _db: &Database) -> Result<(), Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    async fn delete_all(&self, db: &Database) -> Result<(), Box<dyn error::Error>> {
        let collection = self.get_collection(db);
        collection.delete_all().await
    }

    async fn search(&self, db: &Database) -> Result<Vec<Self>, Box<dyn error::Error>> {
        let mut query = doc! { "id": { "$gt": 0 }};
        if self.user_name != "" && self.book_title != "" {
            query = doc! { "$or" : [{"user_id": self.user_id}, {"user_name": {"$regex": &self.user_name}}, {"book_id": &self.book_id}, {"book_title": {"$regex": &self.book_title}}] };
        } else if self.user_name != "" && self.book_title == "" {
            query = doc! { "$or" : [{"user_id": self.user_id}, {"user_name": {"$regex": &self.user_name}}, {"book_id": &self.book_id}] };
        } else if self.user_name == "" && self.book_title != "" {
            query = doc! { "$or" : [{"user_id": self.user_id}, {"book_id": &self.book_id}, {"book_title": {"$regex": &self.book_title}}] };
        } else if self.user_id != 0 || self.book_id != 0 {
            query = doc! { "$or" : [{"user_id": self.user_id}, {"book_id": &self.book_id}] };
        }

        let collection = self.get_collection(db);
        collection.search(query).await
    }

    async fn search_range(
        &self,
        _db: &Database,
        _start_id: u32,
        _end_id: u32,
    ) -> Result<Vec<Self>, Box<dyn error::Error>> {
        panic!("Not implemented")
    }

    fn get_collection_name(&self) -> &str {
        "transactions"
    }
}

#[async_trait]
pub trait HelperCollection<T> {
    async fn update(
        &self,
        query: Document,
        update: Document,
        upsert: bool,
    ) -> Result<(), Box<dyn error::Error>>;
    async fn delete(&self, query: Document) -> Result<(), Box<dyn error::Error>>;
    async fn delete_all(&self) -> Result<(), Box<dyn error::Error>>;
    async fn search(&self, query: Document) -> Result<Vec<T>, Box<dyn error::Error>>;
}

#[async_trait]
impl<T> HelperCollection<T> for Collection<T>
where
    T: DeserializeOwned + Unpin + Send + Sync + Serialize + std::fmt::Debug,
{
    async fn update(
        &self,
        query: Document,
        update: Document,
        upsert: bool,
    ) -> Result<(), Box<dyn error::Error>> {
        let options = FindOneAndUpdateOptions::builder()
            .upsert(upsert)
            .return_document(ReturnDocument::After)
            .build();
        let _ = self.find_one_and_update(query, update, options).await?;
        Ok(())
    }

    async fn delete(&self, query: Document) -> Result<(), Box<dyn error::Error>> {
        let result = self.delete_one(query, None).await?;
        if result.deleted_count == 1 {
            return Ok(());
        } else {
            info!("Data does not eixst");
            return Ok(());
        }
    }

    async fn delete_all(&self) -> Result<(), Box<dyn error::Error>> {
        let options = DropCollectionOptions::builder().build();
        self.drop(options).await?;
        Ok(())
    }

    async fn search(&self, query: Document) -> Result<Vec<T>, Box<dyn error::Error>> {
        let find_options = FindOptions::builder()
            .limit(NUM_SEARCH_ITEMS_MAX)
            .sort(doc! { "id": 1 })
            .build();
        let mut items: Vec<T> = vec![];
        let mut cursor = self.find(query, find_options).await?;
        while let Some(item) = cursor.try_next().await? {
            items.push(item);
        }
        if items.len() == 0 {
            Err(Box::new(Error::new(
                ErrorKind::Other,
                "Item not found".to_string(),
            )))
        } else {
            Ok(items)
        }
    }
}
