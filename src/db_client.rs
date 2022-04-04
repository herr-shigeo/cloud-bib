use crate::error::BibErrorResponse;
use crate::item::create_unique_index;
use actix_web::web;
use async_trait::async_trait;
use bson::Document;
use futures::stream::TryStreamExt;
use log::{debug, info};
use mongodb::bson::doc;
use mongodb::options::{FindOptions, UpdateOptions};
use mongodb::{options::*, Client, Collection, Database};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env;
use std::error;
use std::io::{Error, ErrorKind};
use std::sync::Mutex;

pub struct DbClient {
    connected: bool,
    client: Option<Client>,
}

pub struct DbInstance {
    pub instance: Database,
}

pub async fn get_db(data: &web::Data<Mutex<DbClient>>) -> Result<DbInstance, BibErrorResponse> {
    let db_name =
        env::var("DATABASE_NAME").expect("You must set the DATABSE_NAME environment var!");

    {
        let db_client = data.lock().unwrap();
        if db_client.connected {
            info!("already connected to the db");
            let db = db_client.client.as_ref().unwrap().database(&db_name);
            return Ok(DbInstance { instance: db });
        }
    }

    info!("Started connecting to the db...");
    let new_client = connect()
        .await
        .map_err(|e| BibErrorResponse::DbConnectionError(e.to_string()))?;
    info!("Connected to the db successfully!");

    {
        let mut db_client = data.lock().unwrap();
        db_client.client = Some(new_client.clone());
        db_client.connected = true;
    }

    let db = new_client.database(&db_name);
    let db = DbInstance { instance: db };
    create_unique_index(&db)
        .await
        .map_err(|e| BibErrorResponse::DbConnectionError(e.to_string()))?;

    debug!("Return the db");
    Ok(db)
}

pub fn disconnect_db(data: &web::Data<Mutex<DbClient>>) {
    info!("disconnect_db");
    let mut db_client = data.lock().unwrap();
    db_client.connected = false;
}

async fn connect() -> Result<Client, Box<dyn error::Error>> {
    let client_uri =
        env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");

    let mut client_options = ClientOptions::parse(client_uri).await?;
    let tls_options = TlsOptions::builder().build();
    client_options.tls = Some(Tls::Enabled(tls_options));
    let new_client = Client::with_options(client_options)?;
    Ok(new_client)
}

impl DbClient {
    pub fn new() -> Self {
        Self {
            connected: false,
            client: None,
        }
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
    T: DeserializeOwned + Unpin + Send + Sync + Serialize,
{
    async fn update(
        &self,
        query: Document,
        update: Document,
        upsert: bool,
    ) -> Result<(), Box<dyn error::Error>> {
        debug!("update: {:?}", query);
        let options = UpdateOptions::builder().upsert(upsert).build();
        let result = self.update_one(query, update, options).await?;
        debug!("{:?}", result);
        if result.matched_count == 1 && result.modified_count == 1 && result.upserted_id == None {
            return Ok(());
        }
        if result.matched_count == 0 && result.modified_count == 0 && result.upserted_id != None {
            return Ok(());
        }
        Err(Box::new(Error::new(
            ErrorKind::Other,
            "Update failed for unknwon reason".to_string(),
        )))
    }

    async fn delete(&self, query: Document) -> Result<(), Box<dyn error::Error>> {
        debug!("delete");
        let result = self.delete_one(query, None).await?;
        if result.deleted_count == 1 {
            return Ok(());
        } else {
            panic!("Not implemented")
        }
    }

    async fn delete_all(&self) -> Result<(), Box<dyn error::Error>> {
        let options = DropCollectionOptions::builder().build();
        self.drop(options).await?;
        Ok(())
    }

    async fn search(&self, query: Document) -> Result<Vec<T>, Box<dyn error::Error>> {
        debug!("search: {:?}", query);
        let find_options = FindOptions::builder().sort(doc! { "id": 1 }).build();
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
            debug!("Ok: num = {}", items.len());
            Ok(items)
        }
    }
}
