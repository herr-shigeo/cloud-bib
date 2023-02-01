use chrono::{DateTime, Duration};
use chrono_tz::Tz;
use std::collections::HashMap;
use std::sync::Mutex;

use super::utils::get_nowtime;

#[derive(Clone, Debug)]
pub struct UserToken {
    pub uname: String,
    pub expiration: DateTime<Tz>,
}

impl UserToken {
    pub fn new(uname: String) -> Self {
        let expiration = get_nowtime("Tokyo") + Duration::hours(1);
        Self {
            uname: uname,
            expiration: expiration,
        }
    }
}

pub struct ResetToken {
    pub tokens: Mutex<HashMap<String, UserToken>>,
}

impl ResetToken {
    pub fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, token: String, uname: String) -> Option<UserToken> {
        let mut tokens = self.tokens.lock().unwrap();
        let ret = tokens.insert(token, UserToken::new(uname));
        drop(tokens);

        self.cleanup();
        ret
    }

    pub fn remove(&self, token: &str) -> Option<UserToken> {
        let mut tokens = self.tokens.lock().unwrap();
        let ret = tokens.remove(token);
        drop(tokens);
        match ret {
            Some(ref user_token) => {
                if self.is_expired(&user_token.expiration) {
                    return None;
                }
                return ret;
            }
            None => None,
        }
    }

    fn cleanup(&self) -> () {
        let mut keys_to_remove = vec![];
        let mut tokens = self.tokens.lock().unwrap();
        for (key, value) in tokens.iter() {
            if self.is_expired(&value.expiration) {
                keys_to_remove.push(key.to_owned());
            }
        }
        for key in keys_to_remove {
            tokens.remove(&key);
        }
        drop(tokens);
    }

    fn is_expired(&self, expireation: &DateTime<Tz>) -> bool {
        let now = get_nowtime("Tokyo");
        if now - *expireation > Duration::hours(1) {
            return true;
        } else {
            return false;
        }
    }
}
