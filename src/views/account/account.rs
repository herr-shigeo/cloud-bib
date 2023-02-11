use crate::error::BibErrorResponse;
use crate::item::{
    delete_item, insert_item, search_items, update_item, BarcodeSetting, MonthlyPlan,
    RentalSetting, SystemSetting, SystemUser,
};
use crate::views::cache::Cache;
use crate::views::content_loader::read_file;
use crate::views::db_helper::get_db_with_name;
use crate::views::reply::Reply;
use crate::views::reset_token::ResetToken;
use crate::views::session::{check_admin_session, create_session, get_uname};
use crate::views::transaction::Transaction;
use crate::views::utils::{generate_token, get_system_user, send_email, set_system_limits};
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use argon2::Config;
use rand::Rng;
use serde::Deserialize;
use shared_mongodb::{database, ClientHolder};
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
    static ref DB_COMMON_NAME: String =
        env::var("BIB_DB_NAME").expect("You must set the BIB_DB_NAME environment var!");
}

#[derive(Deserialize, Debug)]
pub struct FormData1 {
    pub uname: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Deserialize, Debug)]
pub struct FormData2 {
    pub uname: String,
    pub email: String,
}

#[derive(Deserialize, Debug)]
pub struct FormData3 {
    pub uname: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Deserialize, Debug)]
pub struct FormData4 {
    pub reset_token: String,
}

pub async fn load_main(_session: Session) -> HttpResponse {
    let html_data = read_file("src/html/account.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

pub async fn load_register(_session: Session) -> HttpResponse {
    let html_data = read_file("src/html/account_register.html").unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

pub async fn add(
    session: Session,
    form: web::Form<FormData1>,
    data: web::Data<Mutex<ClientHolder>>,
    cache_map: web::Data<Mutex<HashMap<String, Cache>>>,
    setting_map: web::Data<Mutex<HashMap<String, SystemSetting>>>,
    transaction_map: web::Data<Mutex<HashMap<String, Transaction>>>,
) -> Result<HttpResponse, BibErrorResponse> {
    let db = get_db_with_name(&data, &DB_COMMON_NAME.to_string()).await?;

    validate_length(&form.uname)?;
    validate_length(&form.email)?;
    validate_length(&form.password)?;

    // Check if the uname already exists
    if get_system_user(&data, Some(form.uname.to_owned()), None)
        .await
        .is_ok()
    {
        return Err(BibErrorResponse::UserExists);
    }

    // Add a user to the DB
    let mut system_user = SystemUser::default();
    system_user.uname = form.uname.to_owned();
    system_user.email = form.email.to_owned();
    system_user.dbname = String::from(&form.uname) + "-system";

    let salt: [u8; 32] = rand::thread_rng().gen();
    let config = Config::default();
    let hashed_password = argon2::hash_encoded(form.password.as_bytes(), &salt, &config)
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    system_user.password = hashed_password.to_owned();
    system_user.operator_password = hashed_password.to_owned();
    system_user.user_password = hashed_password.to_owned();
    system_user.plan = MonthlyPlan::Free;

    insert_item(&db, &system_user)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    // Add a system setting to the DB
    let db = get_db_with_name(&data, &system_user.dbname).await?;
    let mut system_setting = SystemSetting::default();
    system_setting.id = 1;
    set_system_limits(&mut system_setting, &system_user.plan);

    insert_item(&db, &system_setting)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    // Add a rental setting to the DB
    let mut rental_setting = RentalSetting::default();
    rental_setting.id = 1;
    insert_item(&db, &rental_setting)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    // Add a barcode setting to the DB
    let mut barcode_setting = BarcodeSetting::default();
    barcode_setting.id = 1;
    insert_item(&db, &barcode_setting)
        .await
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    // Update the cache_map
    let mut cache_map = cache_map.lock().unwrap();
    let cache = Cache::new();
    cache_map.insert(system_user.dbname.to_owned(), cache);
    drop(cache_map);

    // Update the transaction_map
    let mut transaction_map = transaction_map.lock().unwrap();
    let transaction = Transaction::new(system_setting.max_num_transactions, 0);
    transaction_map.insert(system_user.dbname.to_owned(), transaction);
    drop(transaction_map);

    // Update the setting_map
    let mut setting_map = setting_map.lock().unwrap();
    setting_map.insert(system_user.dbname.to_owned(), system_setting);
    drop(setting_map);

    // create a session
    create_session(
        &session,
        &system_user.uname,
        &system_user.dbname,
        &"admin".to_owned(),
        None,
    )?;

    let mut reply = Reply::default();
    reply.path_to_home = "/account/main".to_owned();

    Ok(HttpResponse::Ok().json(reply))
}

pub async fn update(
    session: Session,
    form: web::Form<FormData2>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_admin_session(&session)?;
    let db = get_db_with_name(&data, &DB_COMMON_NAME.to_string()).await?;

    let uname = get_uname(&session)?;

    let mut system_user = get_system_user(&data, Some(uname), None).await?;
    system_user.email = form.email.to_owned();

    match update_item(&db, &system_user).await {
        Ok(setting) => setting,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

pub async fn delete(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
    cache_map: web::Data<Mutex<HashMap<String, Cache>>>,
    setting_map: web::Data<Mutex<HashMap<String, SystemSetting>>>,
    transaction_map: web::Data<Mutex<HashMap<String, Transaction>>>,
) -> Result<HttpResponse, BibErrorResponse> {
    let dbname = check_admin_session(&session)?;
    let db = get_db_with_name(&data, &DB_COMMON_NAME.to_string()).await?;

    let uname = get_uname(&session)?;

    let mut system_user = SystemUser::default();
    system_user.uname = uname;

    // Delete the user
    match delete_item(&db, &system_user).await {
        Ok(setting) => setting,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    }

    // Delete the DB
    let db = get_db_with_name(&data, &dbname).await?;
    db.drop(None)
        .await
        .map_err(|e| BibErrorResponse::DataNotFound(e.to_string()))?;

    // Delete the cache_map
    let mut cache_map = cache_map.lock().unwrap();
    cache_map.remove(&system_user.dbname);
    drop(cache_map);

    // Delete the transaction_map
    let mut transaction_map = transaction_map.lock().unwrap();
    transaction_map.remove(&system_user.dbname);
    drop(transaction_map);

    // Delete the setting_map
    let mut setting_map = setting_map.lock().unwrap();
    setting_map.remove(&system_user.dbname);
    drop(setting_map);

    // delete the session
    session.purge();

    return Err(BibErrorResponse::NotAuthorized);
}

pub async fn get(
    session: Session,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_admin_session(&session)?;
    let db = get_db_with_name(&data, &DB_COMMON_NAME.to_string()).await?;

    let uname = get_uname(&session)?;
    let system_user = get_system_user(&data, Some(uname), None).await?;

    let mut system_user = match search_items(&db, &system_user).await {
        Ok(system_user) => system_user,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    if system_user.len() != 1 {
        return Err(BibErrorResponse::DataDuplicated(0));
    }
    let system_user = system_user.pop().unwrap();

    let mut reply = Reply::default();
    reply.uname = system_user.uname;
    reply.email = system_user.email;
    reply.plan = system_user.plan.get_str();

    Ok(HttpResponse::Ok().json(reply))
}

pub async fn admin_password(
    session: Session,
    form: web::Form<FormData3>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    log::debug!("{:?}", form);
    check_admin_session(&session)?;
    update_password(&data, &form.uname, "admin", &form.password).await
}

pub async fn operator_password(
    session: Session,
    form: web::Form<FormData3>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_admin_session(&session)?;
    update_password(&data, &form.uname, "operator", &form.password).await
}

pub async fn user_password(
    session: Session,
    form: web::Form<FormData3>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    check_admin_session(&session)?;
    update_password(&data, &form.uname, "user", &form.password).await
}

pub async fn request_reset(
    form: web::Form<FormData2>,
    token_map: web::Data<ResetToken>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    let system_user = get_system_user(&data, Some(form.uname.to_owned()), None).await?;

    if system_user.email != form.email {
        return Err(BibErrorResponse::InvalidArgument(form.email.to_owned()));
    }

    // Gnereate a token
    let token = generate_token();
    token_map.insert(token.to_owned(), system_user.uname);

    // Send an e-mail
    send_email_to_reset(&system_user.email, &token)?;

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

pub async fn prepare_reset(
    form: web::Query<FormData4>,
    token_map: web::Data<ResetToken>,
) -> HttpResponse {
    // check the token
    let token = token_map.remove(&form.reset_token);
    if token.is_none() {
        return HttpResponse::NotFound()
            .content_type("text/html; charset=utf-8")
            .body("ページの有効期限が切れています");
    }

    // load the page
    let mut html_data = read_file("src/html/reset/reset_password.html").unwrap();
    html_data = html_data.replace("{{UNAME}}", &token.unwrap().uname);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_data)
}

pub async fn do_reset(
    form: web::Form<FormData3>,
    data: web::Data<Mutex<ClientHolder>>,
) -> Result<HttpResponse, BibErrorResponse> {
    update_password(&data, &form.uname, "admin", &form.password).await?;
    return Err(BibErrorResponse::NotAuthorized);
}

fn send_email_to_reset(to: &str, token: &str) -> Result<(), BibErrorResponse> {
    let subject = "Reset password for Cloudbib";
    let link = format!(
        "https://www.cloudbib.net/account/prepare_reset?reset_token={}",
        token
    );
    let text = format!("パスワードリセット用のリンクを送ります。\n こちらのリンク先からパスワードをリセットして下さい。 \n{}", link);

    send_email(to, subject, &text).map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;
    Ok(())
}

async fn update_password(
    data: &web::Data<Mutex<ClientHolder>>,
    uname: &str,
    category: &str,
    password: &str,
) -> Result<HttpResponse, BibErrorResponse> {
    let db = get_db_with_name(data, &DB_COMMON_NAME.to_string()).await?;

    validate_length(password)?;

    let mut setting = get_system_user(&data, Some(uname.to_owned()), None).await?;

    // Hash the new password
    let salt: [u8; 32] = rand::thread_rng().gen();
    let config = Config::default();
    let hashed_password = argon2::hash_encoded(password.as_bytes(), &salt, &config)
        .map_err(|e| BibErrorResponse::SystemError(e.to_string()))?;

    match category {
        "admin" => setting.password = hashed_password,
        "operator" => setting.operator_password = hashed_password,
        "user" => setting.user_password = hashed_password,
        _ => {}
    }

    match update_item(&db, &setting).await {
        Ok(setting) => setting,
        Err(e) => {
            database::disconnect(&data);
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

fn validate_length(input: &str) -> Result<&str, BibErrorResponse> {
    if input.len() > 32 {
        return Err(BibErrorResponse::InputLengthTooLong());
    } else {
        return Ok(input);
    }
}
