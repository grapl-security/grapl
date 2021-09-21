use std::ops::Add;

use chrono::{
    Duration,
    Utc,
};
use rand::Rng;
use serde::{
    Deserialize,
    Serialize,
};

use crate::config::{
    SESSION_EXPIRATION_TIMEOUT_DAYS,
    SESSION_TOKEN_LENGTH,
};

#[derive(Serialize, Deserialize, Debug)]
/// Represents a row from the User Session table
pub struct WebSession {
    identity: String,
    token: String,
    session_expiration: i64,
}

impl WebSession {
    pub fn new(identity: &str) -> Self {
        let identity = identity.to_owned();

        let token = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(SESSION_TOKEN_LENGTH)
            .map(char::from)
            .collect::<String>();

        let session_expiration = Utc::now()
            .add(Duration::days(SESSION_EXPIRATION_TIMEOUT_DAYS))
            .timestamp();

        WebSession {
            identity,
            token,
            session_expiration,
        }
    }

    pub fn get_token(&self) -> &str {
        self.token.as_str()
    }

    pub fn get_username(&self) -> &str {
        &self.identity
    }

    pub fn get_session_expiration_timestamp(&self) -> i64 {
        self.session_expiration
    }
}
