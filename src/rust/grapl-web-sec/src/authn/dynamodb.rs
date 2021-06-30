use std::ops::Add;

use argon2::{
    PasswordVerifier,
    Version,
};
use chrono::{
    Duration as ChronoDuration,
    TimeZone,
    Utc,
};
use hmap::hmap;
use rand::Rng;
use rusoto_dynamodb::{
    AttributeValue,
    DynamoDb,
    DynamoDbClient,
    GetItemInput,
    PutItemInput,
};
use serde::{
    Deserialize,
    Serialize,
};
use tap::tap::*;
use tracing::{
    error,
    warn,
};

use crate::error::WebSecError;

const EXPIRATION_TIMEOUT_DAYS: i64 = 1;
const SESSION_TOKEN_LENGTH: usize = 32;

pub(crate) async fn get_valid_session_row(
    dynamodb_client: &DynamoDbClient,
    session_token: String,
) -> Result<SessionRow, WebSecError> {
    let session_instance = hmap! {
        "session_token".to_owned() => AttributeValue {
            s: Some(session_token),
            ..Default::default()
        }
    };

    let session_query = GetItemInput {
        consistent_read: Some(true),
        key: session_instance,
        table_name: grapl_config::user_session_table_name(),
        ..Default::default()
    };

    let session_row_hashmap = dynamodb_client
        .get_item(session_query)
        .await?
        .item
        .ok_or(WebSecError::MissingSession)
        .tap_err(|_| warn!("No entries for session found in session table."))?;

    match serde_dynamodb::from_hashmap::<SessionRow, _>(session_row_hashmap) {
        Err(_) => {
            error!("Failed to deserialize session row data from dynamodb to SessionRow.");
            Err(WebSecError::InvalidSession)
        }
        Ok(session_row) if session_row.is_expired() => {
            // todo: delete session!
            warn!("Session is expired.");
            Err(WebSecError::InvalidSession)
        }
        Ok(session_row) => Ok(session_row),
    }
}

pub(crate) async fn get_user_row(
    dynamodb_client: &DynamoDbClient,
    username: &str,
) -> Result<UserRow, WebSecError> {
    let user_key = hmap! {
        "username".to_owned() => AttributeValue {
            s: Some(username.to_string()),
            ..Default::default()
        }
    };

    let user_query = GetItemInput {
        consistent_read: Some(true),
        key: user_key,
        projection_expression: Some("role".to_string()),
        table_name: grapl_config::user_auth_table_name(),
        ..Default::default()
    };

    let user_row_hashmap = dynamodb_client
        .get_item(user_query)
        .await?
        .item
        .ok_or(WebSecError::InvalidSession)
        .tap_err(|_| {
            error!("Unable to find row in UserAuth table despite user holding valid session.")
        })?;

    serde_dynamodb::from_hashmap::<UserRow, _>(user_row_hashmap)
        .map_err(|_| WebSecError::InvalidSession)
        .tap_err(|_| error!("Unable to deserialize UserRow from UserAuth table."))
}

/// Creates a session if the provided username and password are correct.
///
/// (Session string, expiration UTC i64)
#[allow(dead_code)]
pub(crate) async fn sign_in(
    dynamodb_client: &DynamoDbClient,
    username: &str,
    password: &str,
) -> Result<(String, i64), WebSecError> {
    use argon2::{
        Argon2,
        PasswordHash,
    };

    let user = get_user_row(dynamodb_client, username).await?;
    let password_hash = PasswordHash::new(&user.password_hash)
        .tap_err(|_| {
            error!(
                "Invalid password hash string for user {user}",
                user = username
            )
        })
        .map_err(|e| WebSecError::PasswordHashingError(e))?;

    // IMPORTANT: Keep in sync w/ https://github.com/grapl-security/grapl/blob/main/src/python/provisioner/provisioner/app.py#L84
    let password_hasher = Argon2::new(None, 2, 102400, 8, Version::V0x13)
        .map_err(|e| WebSecError::ArgonError(e))
        .tap_err(|_| {
            error!("Failed to create Argon password hasher due to invalid parameters!!!")
        })?;

    // Verify the password against the password hash (which contains a user-specific salt and configuration information)
    password_hasher
        .verify_password(password.as_bytes(), &password_hash)
        .tap_err(|_| warn!("Incorrect password."))
        .map_err(|e| WebSecError::PasswordHashingError(e))?;

    // create a session
    let expiry = Utc::now().add(ChronoDuration::days(EXPIRATION_TIMEOUT_DAYS));
    let session_string = create_session(dynamodb_client, username, expiry.timestamp())
        .await
        .tap_err(|_| {
            error!(
                "Failed to write a new session to dynamodb for user {user}",
                user = username
            )
        })?;

    Ok((session_string, expiry.timestamp()))
}

async fn create_session(
    dynamodb_client: &DynamoDbClient,
    username: &str,
    expiration: i64,
) -> Result<String, WebSecError> {
    let session_token = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(SESSION_TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>();

    let session_entry = hmap! {
        "session_token".to_owned() => AttributeValue {
            s: Some(session_token.clone()),
            ..Default::default()
        },
        "expiration".to_owned() => AttributeValue {
            n: Some(expiration.to_string()),
            ..Default::default()
        },
        "username".to_owned() => AttributeValue {
            s: Some(username.to_owned()),
            ..Default::default()
        }
    };

    let new_session_input = PutItemInput {
        item: session_entry,
        table_name: grapl_config::user_session_table_name(),
        ..Default::default()
    };

    dynamodb_client
        .put_item(new_session_input)
        .await
        .map_err(|_| WebSecError::SessionCreationError)?;

    Ok(session_token)
}

#[derive(Serialize, Deserialize)]
/// Represents a row from the User Session table
pub(crate) struct SessionRow {
    pub(crate) username: String,
    pub(crate) expiration: i64,
}

impl SessionRow {
    fn is_expired(&self) -> bool {
        let expiration = Utc.timestamp(self.expiration, 0);

        Utc::now().ge(&expiration)
    }
}

/// Represents a row from the User Auth table
#[derive(Deserialize)]
pub(crate) struct UserRow {
    pub(crate) role: GraplRole,
    password_hash: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GraplRole {
    Owner,
    Administrator,
    User,
}

impl ToString for GraplRole {
    fn to_string(&self) -> String {
        match self {
            GraplRole::Owner => "owner".to_string(),
            GraplRole::Administrator => "administrator".to_string(),
            GraplRole::User => "user".to_string(),
        }
    }
}
