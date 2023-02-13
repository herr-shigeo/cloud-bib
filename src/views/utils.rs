use std::{env, sync::Mutex};

use actix_web::web;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{Asia::Tokyo, Europe::Berlin, Tz};
extern crate lettre;
extern crate lettre_email;
use lettre::{smtp::authentication::IntoCredentials, SmtpClient, Transport};
use lettre_email::EmailBuilder;
use log::debug;
use mongodb::Database;
use select::document::Document;
use select::predicate::Name;
use shared_mongodb::ClientHolder;
use uuid::Uuid;

use lazy_static::lazy_static;

use crate::{
    error::BibErrorResponse,
    item::{search_items, update_item, Book, MonthlyPlan, SystemSetting, SystemUser},
};

use super::{constatns::*, db_helper::get_db_with_name};

lazy_static! {
    static ref DB_COMMON_NAME: String =
        env::var("BIB_DB_NAME").expect("You must set the BIB_DB_NAME environment var!");
    static ref EMAIL_SMTP_RELAY: String =
        env::var("EMAIL_SMTP_RELAY").expect("You must set the EMAIL_SMTP_RELAY environment var!");
    static ref EMAIL_USER: String =
        env::var("EMAIL_USER").expect("You must set the EMAIL_USER environment var!");
    static ref EMAIL_FROM: String =
        env::var("EMAIL_FROM").expect("You must set the EMAIL_EMAIL_FROM environment var!");
    static ref EMAIL_PASSWORD: String =
        env::var("EMAIL_PASSWORD").expect("You must set the EMAIL_PASSWORD environment var!");
}

pub async fn fetch_book_info(isbn: &str) -> Result<Book, BibErrorResponse> {
    let url = format!("https://iss.ndl.go.jp/api/opensearch?isbn={}", isbn);
    let res = reqwest::get(&url)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    if !res.status().is_success() {
        return Err(BibErrorResponse::SystemError(res.status().to_string()));
    }

    let body = res.text().await.unwrap();
    let document = Document::from(body.as_str());

    let title = document
        .find(Name("title"))
        .skip(1)
        .next()
        .map_or("".to_string(), |node| node.text());

    let author = document
        .find(Name("author"))
        .next()
        .map_or("".to_string(), |node| node.text());

    let title_kana = document
        .find(Name("dcndl:titletranscription"))
        .next()
        .map_or("".to_string(), |node| node.text());

    let series = document
        .find(Name("dcndl:seriestitle"))
        .next()
        .map_or("".to_string(), |node| node.text());

    let publisher = document
        .find(Name("dc:publisher"))
        .next()
        .map_or("".to_string(), |node| node.text());

    let published_date = document
        .find(Name("dc:date"))
        .next()
        .map_or("".to_string(), |node| node.text());

    let page = document
        .find(Name("dc:extent"))
        .next()
        .map_or("".to_string(), |node| node.text());

    let mut book = Book::default();
    book.title = title;
    book.kana = title_kana;
    book.author = author;
    book.publisher = publisher;
    book.published_date = published_date;
    book.series = series;
    book.page = page;

    book.isbn = isbn.to_owned();

    Ok(book)
}

pub fn generate_token() -> String {
    Uuid::new_v4().to_string()
}

pub fn get_nowtime(time_zone: &str) -> DateTime<Tz> {
    let utc = Utc::now().naive_utc();

    if "Berlin".eq(time_zone) {
        return Berlin.from_utc_datetime(&utc);
    } else {
        return Tokyo.from_utc_datetime(&utc);
    }
}

pub fn send_email(to: &str, subject: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let smtp_address: &str = &EMAIL_SMTP_RELAY;
    let username: &str = &EMAIL_USER;
    let email_from: &str = &EMAIL_FROM;
    let password: &str = &EMAIL_PASSWORD;
    let email = EmailBuilder::new()
        .from(email_from)
        .to(to)
        .subject(subject)
        .text(text)
        .build()
        .unwrap()
        .into();
    let credentials = (username, password).into_credentials();
    let mut client = SmtpClient::new_simple(smtp_address)
        .unwrap()
        .credentials(credentials)
        .transport();
    let _result = client.send(email);
    Ok(())
}

pub fn set_system_limits(system_setting: &mut SystemSetting, plan: &MonthlyPlan) -> () {
    match plan {
        MonthlyPlan::Free => {
            system_setting.max_registered_users = NUM_USERS_FOR_FREE;
            system_setting.max_registered_books = NUM_BOOKS_FOR_FREE;
            system_setting.max_num_transactions = NUM_TRANSACTIONS_FOR_FREE;
        }
        MonthlyPlan::Light => {
            system_setting.max_registered_users = NUM_USERS_FOR_LIGHT;
            system_setting.max_registered_books = NUM_BOOKS_FOR_LIGHT;
            system_setting.max_num_transactions = NUM_TRANSACTIONS_FOR_LIGHT;
        }
        MonthlyPlan::Standard => {
            system_setting.max_registered_users = NUM_USERS_FOR_STANDARD;
            system_setting.max_registered_books = NUM_BOOKS_FOR_STANDARD;
            system_setting.max_num_transactions = NUM_TRANSACTIONS_FOR_STANDARD;
        }
    };
}

pub async fn update_subscription(
    data: &web::Data<Mutex<ClientHolder>>,
    uname: &str,
    plan: &MonthlyPlan,
    subscription_id: String,
) -> Result<SystemSetting, BibErrorResponse> {
    // Update the plan and subscription id
    let mut system_user = get_system_user(data, Some(uname.to_owned()), None).await?;
    system_user.plan = plan.to_owned();
    match system_user.plan {
        MonthlyPlan::Free => system_user.subscription_id = String::new(),
        _ => {
            if system_user.subscription_id != "" && system_user.subscription_id != subscription_id {
                log::warn!(
                    "The subscription id does not match, {} != {}",
                    system_user.subscription_id,
                    subscription_id
                );
            }
            system_user.subscription_id = subscription_id;
        }
    }

    debug!("update_subscription: {:?}", system_user);

    let db = get_db_with_name(data, &DB_COMMON_NAME.to_string()).await?;
    update_item(&db, &system_user)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    // Update the sytem setting
    let db = get_db_with_name(&data, &system_user.dbname).await?;
    update_system_setting(&db, plan).await
}

pub async fn get_system_user(
    data: &web::Data<Mutex<ClientHolder>>,
    uname: Option<String>,
    subscription_id: Option<String>,
) -> Result<SystemUser, BibErrorResponse> {
    let db = get_db_with_name(data, &DB_COMMON_NAME.to_string()).await?;
    let mut system_user = SystemUser::default();
    if uname.is_some() {
        system_user.uname = uname.unwrap();
    } else if subscription_id.is_some() {
        system_user.subscription_id = subscription_id.unwrap();
    }

    let mut system_user = match search_items(&db, &system_user).await {
        Ok(system_user) => system_user,
        Err(e) => {
            debug!(
                "get_system-user(uname={}, subscription_id={}) returns {:?}",
                system_user.uname, system_user.subscription_id, e
            );
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    if system_user.len() == 1 {
        return Ok(system_user.pop().unwrap());
    } else {
        return Err(BibErrorResponse::DataDuplicated(
            system_user.len().try_into().unwrap(),
        ));
    }
}

async fn update_system_setting(
    db: &Database,
    plan: &MonthlyPlan,
) -> Result<SystemSetting, BibErrorResponse> {
    // Read the system setting
    let mut system_setting = SystemSetting::default();
    system_setting.id = 1;
    let mut system_setting = match search_items(&db, &system_setting).await {
        Ok(system_setting) => system_setting,
        Err(e) => {
            debug!("update_system_setting returns {:?}", e);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };
    if system_setting.len() != 1 {
        return Err(BibErrorResponse::DataDuplicated(
            system_setting.len().try_into().unwrap(),
        ));
    }
    let mut system_setting = system_setting.pop().unwrap();

    set_system_limits(&mut system_setting, plan);

    // Write back
    update_item(&db, &system_setting)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    Ok(system_setting)
}
