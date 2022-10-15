use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{Asia::Tokyo, Europe::Berlin, Tz};
use log::debug;

pub fn get_nowtime(time_zone: &str) -> DateTime<Tz> {
    let utc = Utc::now().naive_utc();
    debug!("time_zone = {}", time_zone);

    if "Berlin".eq(time_zone) {
        return Berlin.from_utc_datetime(&utc);
    } else {
        return Tokyo.from_utc_datetime(&utc);
    }
}
