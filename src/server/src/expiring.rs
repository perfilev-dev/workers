use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::time::{Duration, SystemTime};

#[derive(Serialize, Deserialize)]
pub struct ExpiringData {
    pub data: String,
    pub expires_on: SystemTime,
}

impl ExpiringData {
    pub fn new(data: &str, ttl: Duration) -> ExpiringData {
        ExpiringData {
            data: data.to_string(),
            expires_on: SystemTime::now().add(ttl),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_on < SystemTime::now()
    }
}
