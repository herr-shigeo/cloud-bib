use actix_web::{web, App, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct StripeSession {
    id: String,
    object: String,
    api_version: String,
    created: u64,
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
    Delete {
        metadata: Metadata,
    },
}

#[derive(Deserialize, Debug)]
struct CustomerDetails {
    email: String,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    plan: String,
}

impl StripeSession {
    fn new(json_str: &str) -> Result<StripeSession, serde_json::Error> {
        let event: StripeSession = serde_json::from_str(json_str)?;
        Ok(event)
    }
}

pub async fn subscription_deleted(
    json: web::Json<serde_json::Value>,
) -> actix_web::Result<actix_web::HttpResponse> {
    log::debug!("subscription_deleted");
    Ok(actix_web::HttpResponse::Ok().body("Success"))
}

pub async fn subscription_updated(
    json: web::Json<serde_json::Value>,
) -> actix_web::Result<actix_web::HttpResponse> {
    log::debug!("subscription_updated");
    Ok(actix_web::HttpResponse::Ok().body("Success"))
}

pub async fn checkout_completed(
    json: web::Json<serde_json::Value>,
) -> actix_web::Result<actix_web::HttpResponse> {
    let data = json.into_inner();

    log::debug!("{:?}", data);

    let json_str = serde_json::to_string(&data).unwrap();
    let session = match StripeSession::new(&json_str) {
        Ok(session) => session,
        Err(error) => {
            log::error!("Failed to parse Stripe event: {}", error);
            return Ok(actix_web::HttpResponse::BadRequest().body("Failed to parse Stripe event"));
        }
    };

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
        }
        StripeEvent::Delete { .. } => {
            // handle the Delete event here
        }
    }

    Ok(actix_web::HttpResponse::Ok().body("Success"))
}
