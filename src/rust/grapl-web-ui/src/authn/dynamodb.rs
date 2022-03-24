use chrono::{
    TimeZone,
    Utc,
};
use hmap::hmap;
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

use super::{
    GraplRole,
    Password,
    WebSession,
};

pub struct AuthDynamoClient {
    pub client: DynamoDbClient,
    pub user_auth_table_name: String,
    pub user_session_table_name: String,
}

type SessionToken = String;
type Username = String;

#[derive(thiserror::Error, Debug)]
pub enum AuthDynamoClientError {
    #[error("user not found: `{0}`")]
    UserRecordNotFound(Username),
    #[error("unable to get item from DynamoDB: {0}")]
    GetItem(#[from] rusoto_core::RusotoError<rusoto_dynamodb::GetItemError>),
    #[error("unable to put item in DynamoDB: {0}")]
    PutItem(#[from] rusoto_core::RusotoError<rusoto_dynamodb::PutItemError>),
    #[error("unable to deserialize response from DynamoDB: {0}")]
    Parsing(#[from] serde_dynamodb::Error),
    #[error("unable to verify password for user `{username}`: {source}")]
    PasswordVerification {
        username: String,
        source: argon2::password_hash::Error,
    },
}

impl AuthDynamoClient {
    #[tracing::instrument(err, skip(self, token))]
    pub(crate) async fn get_valid_session_row(
        &self,
        token: SessionToken,
    ) -> Result<Option<SessionRow>, AuthDynamoClientError> {
        let session_instance = hmap! {
            "session_token".to_owned() => AttributeValue {
                s: Some(token),
                ..Default::default()
            }
        };

        let session_query = GetItemInput {
            consistent_read: Some(true),
            key: session_instance,
            table_name: self.user_session_table_name.clone(),
            ..Default::default()
        };

        // do not log session_query here
        tracing::debug!("Getting user session from DynamoDB.");

        let session_row_hashmap = self.client.get_item(session_query).await?.item;
        let session_row_hashmap = match session_row_hashmap {
            Some(session_row_hashmap) => session_row_hashmap,
            None => {
                tracing::debug!("Session not found in database");

                return Ok(None);
            }
        };

        tracing::debug!("Got user session from DynamoDB database.");

        let session_row = serde_dynamodb::from_hashmap::<SessionRow, _>(session_row_hashmap)?;

        if session_row.is_expired() {
            // todo: delete session!
            // note(inickles): i think we should rely on dynamo ttl to autodelete, which will
            // clean up more than us doing it here. specfically: a user session that expires
            // and where a request for that session is never made again - the entry would never
            // be removed from the database. however, it also means session length is
            // not configurable here, but on the DB table itself.
            tracing::debug!(message = "session expired", username = ?session_row.username, expiration = ?session_row.expiration);

            Ok(None)
        } else {
            Ok(Some(session_row))
        }
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) async fn get_user_row(
        &self,
        username: &str,
    ) -> Result<UserRow, AuthDynamoClientError> {
        let user_key = hmap! {
            "username".to_owned() => AttributeValue {
                s: Some(username.to_string()),
                ..Default::default()
            }
        };

        let user_query = GetItemInput {
            consistent_read: Some(true),
            key: user_key,
            // projection_expression: Some("grapl_role".to_string()),
            table_name: self.user_auth_table_name.clone(),
            ..Default::default()
        };

        tracing::debug!(
            message = "Getting user record from DynamoDB.",
            query = ?user_query
        );

        let user_row_hashmap = self.client.get_item(user_query).await?.item.ok_or(
            AuthDynamoClientError::UserRecordNotFound(username.to_string()),
        )?;

        // do not log the database repsonse, it probably contains sensitive information like user
        // password hash.
        tracing::debug!("Got user record from DynamoDB database.");

        Ok(serde_dynamodb::from_hashmap::<UserRow, _>(
            user_row_hashmap,
        )?)
    }

    #[tracing::instrument(err, skip(self, password))]
    pub(crate) async fn sign_in(
        &self,
        username: &str,
        password: &Password,
    ) -> Result<WebSession, AuthDynamoClientError> {
        let user = self.get_user_row(username).await?;

        // Verify the supplied password against hash in the database
        password
            .verify_hash(&user.password_hash)
            .map_err(|source| AuthDynamoClientError::PasswordVerification {
                username: username.to_string(),
                source,
            })?;

        let session = WebSession::new(username);
        self.store_session(&session).await?;

        Ok(session)
    }

    #[tracing::instrument(err, skip(self))]
    async fn store_session(&self, session: &WebSession) -> Result<(), AuthDynamoClientError> {
        let session_entry = hmap! {
            "session_token".to_owned() => AttributeValue {
                s: Some(session.get_token().to_owned()),
                ..Default::default()
            },
            "expiration".to_owned() => AttributeValue {
                n: Some(session.get_session_expiration_timestamp().to_string()),
                ..Default::default()
            },
            "username".to_owned() => AttributeValue {
                s: Some(session.get_username().to_owned()),
                ..Default::default()
            }
        };

        let new_session_input = PutItemInput {
            item: session_entry,
            table_name: self.user_session_table_name.clone(),
            ..Default::default()
        };

        tracing::debug!(
            message = "Adding user session to DynamoDB",
            item =? new_session_input
        );

        self.client.put_item(new_session_input).await?;

        tracing::debug!("User session successfully added.");

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
/// Represents a row from the User Session table
pub(crate) struct SessionRow {
    pub(crate) username: String,
    pub(crate) expiration: i64,
}

//TODO(inickles): reconcile this with WebSession
impl SessionRow {
    fn is_expired(&self) -> bool {
        let expiration = Utc.timestamp(self.expiration, 0);

        Utc::now().ge(&expiration)
    }
}

/// Represents a row from the User Auth table
#[derive(Deserialize)]
pub(crate) struct UserRow {
    pub(crate) grapl_role: GraplRole,
    password_hash: String,
}
