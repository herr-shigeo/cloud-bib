use crate::item::*;
use chrono::{DateTime, Duration};
use chrono_tz::Tz;
use std::collections::HashMap;
use std::sync::Mutex;

use super::utils::get_nowtime;

pub struct ResetToken {
    pub tokens: Mutex<HashMap<String, DateTime<Tz>>>,
}

impl ResetToken {
    pub fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, token: String) -> Option<DateTime<Tz>> {
        let expiration = get_nowtime("Tokyo") + Duration::hours(1);

        let mut tokens = self.tokens.lock().unwrap();
        let ret = tokens.insert(token, expiration);
        drop(tokens);
        ret
    }

    pub fn remove(&self, token: &str) -> Option<DateTime<Tz>> {
        let mut tokens = self.tokens.lock().unwrap();
        let ret = tokens.remove(token);
        drop(tokens);
        ret
    }
}
