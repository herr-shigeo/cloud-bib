use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{Asia::Tokyo, Europe::Berlin, Tz};
use log::debug;
use select::{document::Document, predicate::Name};

use crate::{error::BibErrorResponse, item::Book};

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

    let mut book = Book::default();
    book.title = title;
    book.kana = title_kana;
    book.author = author;
    book.publisher = publisher;
    book.series = series;

    Ok(book)
}

pub fn get_nowtime(time_zone: &str) -> DateTime<Tz> {
    let utc = Utc::now().naive_utc();
    debug!("time_zone = {}", time_zone);

    if "Berlin".eq(time_zone) {
        return Berlin.from_utc_datetime(&utc);
    } else {
        return Tokyo.from_utc_datetime(&utc);
    }
}
