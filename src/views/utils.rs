use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{Asia::Tokyo, Europe::Berlin, Tz};
extern crate lettre;
extern crate lettre_email;
use lettre::smtp::authentication::IntoCredentials;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
use uuid::Uuid;

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

pub fn test() -> Result<(), Box<dyn std::error::Error>> {
    let smtp_address = "smtp-relay.sendinblue.com";
    let username = "cloudbib.info@gmail.com";
    let password = "HkdVn0phYm8Lg1bj";
    let email = EmailBuilder::new()
        .to("cloudbib.info@gmail.com")
        .from(username)
        .subject("Which bear is best?")
        .text("Bears eat beets. Bears. Beets. Battlestar Galactica.")
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
