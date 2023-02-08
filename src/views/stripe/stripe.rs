use crate::{
    error::BibErrorResponse,
    item::{delete_item_all, Book, MonthlyPlan, SystemSetting, TransactionItem, User},
    views::{
        cache::Cache,
        constatns::{PRICE_ID_FOR_LIGHT, PRICE_ID_FOR_STANDARD},
        db_helper::get_db_with_name,
        reply::Reply,
        transaction::Transaction,
        utils::{get_system_user, update_subscription},
    },
};
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use shared_mongodb::ClientHolder;
use std::{collections::HashMap, env, str::FromStr, sync::Mutex};

use lazy_static::lazy_static;

lazy_static! {
    static ref DB_COMMON_NAME: String =
        env::var("BIB_DB_NAME").expect("You must set the BIB_DB_NAME environment var!");
}

#[derive(Deserialize, Debug)]
pub struct StripeSession {
    data: StripeSessionData,
}

#[derive(Deserialize, Debug)]
pub struct StripeSessionData {
    object: StripeEvent,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum StripeEvent {
    Checkout {
        client_reference_id: String,
        customer_details: CustomerDetails,
        metadata: Metadata,
        subscription: String,
    },
    UpdateOrDelete {
        id: String, // subscription
        items: SubscriptionItems,
        status: String,
    },
}
#[derive(Deserialize, Debug)]
pub struct SubscriptionItems {
    data: Vec<SubscriptionData>,
}

#[derive(Deserialize, Debug)]
pub struct SubscriptionData {
    plan: SubscriptionPlan,
}

#[derive(Deserialize, Debug)]
pub struct SubscriptionPlan {
    id: String,
}

#[derive(Deserialize, Debug)]
pub struct CustomerDetails {
    email: String,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    plan: String,
}

impl StripeSession {
    fn new(json_str: &str) -> Result<StripeSession, serde_json::Error> {
        let event: StripeSession = serde_json::from_str(json_str)?;
        Ok(event)
    }
}

pub async fn webhook(
    json: web::Json<serde_json::Value>,
    data: web::Data<Mutex<ClientHolder>>,
    cache_map: web::Data<Mutex<HashMap<String, Cache>>>,
    setting_map: web::Data<Mutex<HashMap<String, SystemSetting>>>,
    transaction_map: web::Data<Mutex<HashMap<String, Transaction>>>,
) -> Result<HttpResponse, BibErrorResponse> {
    let json_data = json.into_inner();

    let json_str = serde_json::to_string(&json_data).unwrap();
    let session = match StripeSession::new(&json_str) {
        Ok(session) => session,
        Err(error) => {
            log::error!("Failed to parse Stripe event: {}", error);
            return Ok(actix_web::HttpResponse::BadRequest().body("Failed to parse Stripe event"));
        }
    };

    log::debug!("webhook");

    let mut plan: Option<MonthlyPlan> = None;
    let subscription_id: Option<String>;
    let system_user;

    match session.data.object {
        StripeEvent::Checkout {
            client_reference_id,
            customer_details,
            metadata,
            subscription,
            ..
        } => {
            log::debug!("client_reference_id: {}", client_reference_id);
            log::debug!("email: {}", customer_details.email);
            log::debug!("plan: {}", metadata.plan);
            log::debug!("subscription: {}", subscription);

            // Search a user
            system_user =
                get_system_user(&data, Some(client_reference_id.to_owned()), None).await?;

            plan = MonthlyPlan::from_str(&metadata.plan).map_or(None, |plan| Some(plan));
            subscription_id = Some(subscription);
        }
        StripeEvent::UpdateOrDelete {
            id,
            mut items,
            status,
            ..
        } => {
            log::debug!("{}:({})", id, status);

            // Search a user
            system_user = get_system_user(&data, None, Some(id.to_owned())).await?;

            subscription_id = Some(id);

            if status == "canceled" {
                plan = Some(MonthlyPlan::Free);
            } else if status == "active" {
                if items.data.len() == 1 {
                    let price_id = items.data.pop().unwrap().plan.id;
                    plan = get_plan_from_id(&price_id);
                }
            } else {
                return Err(BibErrorResponse::SystemError(format!(
                    "Unknown status: {}",
                    status
                )));
            }
        }
    }

    if plan.is_none() {
        return Err(BibErrorResponse::SystemError("plan is unknown".to_string()));
    }
    let plan = plan.unwrap();

    if subscription_id.is_none() {
        return Err(BibErrorResponse::SystemError(
            "subscription id is unknown".to_string(),
        ));
    }
    let subscription_id = subscription_id.unwrap();

    // Update the subscription
    let mut setting_map = setting_map.lock().unwrap();

    let system_setting =
        update_subscription(&data, &system_user.uname, &plan, subscription_id).await?;

    setting_map.insert(system_user.dbname.to_owned(), system_setting.to_owned());
    drop(setting_map);

    // Clear the DB if the plan is downgraded
    if plan.is_downgraded(&system_user.plan) {
        let db = get_db_with_name(&data, &system_user.dbname).await?;
        let user = User::default();
        let _ = delete_item_all(&db, &user)
            .await
            .map_err(|e| log::error!("{:?}", e));

        let book = Book::default();
        let _ = delete_item_all(&db, &book)
            .await
            .map_err(|e| log::error!("{:?}", e));

        let transaction = TransactionItem::default();
        let _ = delete_item_all(&db, &transaction)
            .await
            .map_err(|e| log::error!("{:?}", e));

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
    }

    let reply = Reply::default();
    Ok(HttpResponse::Ok().json(reply))
}

fn get_plan_from_id(price_id: &str) -> Option<MonthlyPlan> {
    match price_id {
        PRICE_ID_FOR_LIGHT => Some(MonthlyPlan::Light),
        PRICE_ID_FOR_STANDARD => Some(MonthlyPlan::Standard),
        _ => None,
    }
}
