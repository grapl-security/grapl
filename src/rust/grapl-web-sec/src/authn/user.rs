use actix_web::{FromRequest, HttpRequest};
use std::pin::Pin;
use futures::Future;
use actix_web::dev::Payload;
use grapl_config::env_helpers::FromEnv;
use rusoto_dynamodb::DynamoDbClient;
use crate::error::WebSecError;
use crate::authn::GraplRole;

const SESSION_TOKEN: &'static str = "SESSION_TOKEN";

/// Represents an Authenticated User
#[derive(Clone)]
pub struct AuthenticatedUser {
    identity: String,
    role: GraplRole
}

impl AuthenticatedUser {
    pub fn get_identity(&self) -> &str {
        &self.identity
    }

    pub fn get_role(&self) -> &GraplRole {
        &self.role
    }
}

impl AuthenticatedUser {
    #[allow(dead_code)]
    #[cfg(test)]
    /// Used only for constructing tests; should NEVER be used in production code
    pub fn test_user(identity: &str, role: GraplRole) -> Self {
        Self {
            identity: identity.to_string(),
            role
        }
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = WebSecError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        use actix_session::UserSession;
        use crate::authn::dynamodb;

        let session_storage = req.get_session();

        let attempt_authentication = async move {
            let dynamodb_client = DynamoDbClient::from_env();

            let session_token: String = session_storage.get(SESSION_TOKEN)
                .map_err(|_| WebSecError::MissingSession)?
                .ok_or(WebSecError::MissingSession)?;

            let session_row = dynamodb::get_valid_session_row(&dynamodb_client, session_token).await?;
            let user_row = dynamodb::get_user_row(&dynamodb_client, &session_row.username).await?;

            let authenticated_user = AuthenticatedUser {
                identity: session_row.username,
                role: user_row.role
            };

            Ok(authenticated_user)
        };

        Box::pin(attempt_authentication)
    }
}