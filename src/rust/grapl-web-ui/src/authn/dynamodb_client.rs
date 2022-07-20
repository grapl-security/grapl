use hmap::hmap;
use rusoto_dynamodb::{
    AttributeValue,
    DynamoDb,
    GetItemInput,
    PutItemInput,
};
use secrecy::ExposeSecret;

use super::{
    GraplRole,
    Secret,
};

pub(crate) struct AuthDynamoClient {
    client: rusoto_dynamodb::DynamoDbClient,
    user_auth_table_name: String,
    user_session_table_name: String,
}

type Result<T> = std::result::Result<T, AuthDynamoClientError>;
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum AuthDynamoClientError {
    #[error("unable to get item from DynamoDB: {0}")]
    GetItem(#[from] rusoto_core::RusotoError<rusoto_dynamodb::GetItemError>),
    #[error("unable to put item in DynamoDB: {0}")]
    PutItem(#[from] rusoto_core::RusotoError<rusoto_dynamodb::PutItemError>),
    #[error("unable to deserialize response from DynamoDB: {0}")]
    Parsing(#[from] serde_dynamodb::Error),
}

impl AuthDynamoClient {
    pub(crate) fn new(
        client: rusoto_dynamodb::DynamoDbClient,
        user_auth_table_name: String,
        user_session_table_name: String,
    ) -> Self {
        AuthDynamoClient {
            client,
            user_auth_table_name,
            user_session_table_name,
        }
    }

    pub(super) async fn get_user_row(&self, username: String) -> Result<Option<UserRow>> {
        let user_key = hmap! {
            "username".to_owned() => AttributeValue {
                s: Some(username),
                ..Default::default()
            }
        };

        let user_query = GetItemInput {
            consistent_read: Some(true),
            key: user_key,
            table_name: self.user_auth_table_name.clone(),
            ..Default::default()
        };

        tracing::debug!(
            message = "Getting user record from DynamoDB.",
            query = ?user_query
        );

        self.client
            .get_item(user_query)
            .await?
            .item
            .map(serde_dynamodb::from_hashmap::<UserRow, _>)
            .transpose()
            .map_err(|e| e.into())
    }

    /// Returns web session information if found and if the session has not expired.
    pub(super) async fn get_web_session_row(
        &self,
        token: Secret<String>,
    ) -> Result<Option<WebSessionRow>> {
        let session_key = hmap! {
            "session_token".to_owned() => AttributeValue {
                s: Some(token.expose_secret().to_owned()),
                ..Default::default()
            }
        };

        let session_query = GetItemInput {
            consistent_read: Some(true),
            key: session_key,
            table_name: self.user_session_table_name.clone(),
            ..Default::default()
        };

        // do not log session_query here
        tracing::debug!("Getting user session from DynamoDB.");

        self.client
            .get_item(session_query)
            .await?
            .item
            .map(serde_dynamodb::from_hashmap::<WebSessionRow, _>)
            .transpose()
            .map_err(|e| e.into())
    }

    pub(super) async fn store_web_session(
        &self,
        username: String,
        token: String,
        expiration: i64,
    ) -> Result<()> {
        let session_entry = hmap! {
            "username".to_owned() => AttributeValue {
                s: Some(username),
                ..Default::default()
            },
            "session_token".to_owned() => AttributeValue {
                s: Some(token),
                ..Default::default()
            },
            "expiration".to_owned() => AttributeValue {
                n: Some(expiration.to_string()),
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

/// Represents that was retrieved from the user authentication table
#[derive(serde::Deserialize)]
pub(super) struct UserRow {
    username: String,
    grapl_role: GraplRole,
    password_hash: Secret<String>,
    organization_id: String,
}

impl UserRow {
    pub(super) fn get_username(&self) -> &str {
        self.username.as_str()
    }

    pub(super) fn get_role(&self) -> &GraplRole {
        &self.grapl_role
    }

    pub(super) fn get_password_hash(&self) -> &Secret<String> {
        &self.password_hash
    }

    pub(super) fn get_organization_id(&self) -> &str {
        self.organization_id.as_str()
    }
}

#[derive(serde::Deserialize, Debug)]
/// Represents a row from the User Session table
pub struct WebSessionRow {
    username: String,
    expiration: i64,
}

impl WebSessionRow {
    pub(super) fn get_username(&self) -> &str {
        &self.username
    }

    pub(super) fn get_expiration(&self) -> i64 {
        self.expiration
    }
}
