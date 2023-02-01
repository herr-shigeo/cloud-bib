use std::env;

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{Asia::Tokyo, Europe::Berlin, Tz};
extern crate lettre;
extern crate lettre_email;
use lettre::{smtp::authentication::IntoCredentials, SmtpClient, Transport};
use lettre_email::EmailBuilder;
use uuid::Uuid;

use lazy_static::lazy_static;

lazy_static! {
    static ref EMAIL_SMTP_RELAY: String =
        env::var("EMAIL_SMTP_RELAY").expect("You must set the EMAIL_SMTP_RELAY environment var!");
    static ref EMAIL_USER: String =
        env::var("EMAIL_USER").expect("You must set the EMAIL_USER environment var!");
    static ref EMAIL_FROM: String =
        env::var("EMAIL_FROM").expect("You must set the EMAIL_EMAIL_FROM environment var!");
    static ref EMAIL_PASSWORD: String =
        env::var("EMAIL_PASSWORD").expect("You must set the EMAIL_PASSWORD environment var!");
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
