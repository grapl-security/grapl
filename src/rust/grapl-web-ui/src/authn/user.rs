use std::pin::Pin;

use actix_session::UserSession;
use actix_web::{
    dev::Payload,
    http::StatusCode,
    FromRequest,
    HttpRequest,
    ResponseError,
};
use futures_util::future::Future;
use tracing::Instrument;

use crate::authn::{
    AuthDynamoClientError,
    GraplRole,
};

/// Represents an Authenticated User
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    identity: String,
    role: GraplRole,
}

#[derive(thiserror::Error, Debug)]
pub enum UserAuthenticationError {
    #[error(transparent)]
    Client(#[from] AuthDynamoClientError),
    #[error(transparent)]
    Cookie(#[from] actix_web::Error),
    #[error("Session token not found in user cookie")]
    NoSessionToken,
    #[error("Unable to get database client from Actix app data")]
    AddData,
    // This Opaque variant is a hack around mixing tokio runtimes and
    // ResponseError not implementing Send
    #[error("Opaque error from DB client")]
    Opaque,
}

impl ResponseError for UserAuthenticationError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

impl AuthenticatedUser {
    pub fn get_identity(&self) -> &str {
        &self.identity
    }

    // pub fn get_role(&self) -> &GraplRole {
    //     &self.role
    // }
}

impl AuthenticatedUser {
    #[allow(dead_code)]
    #[cfg(test)]
    /// Used only for constructing tests; should NEVER be used in production code
    pub fn test_user(identity: &str, role: GraplRole) -> Self {
        Self {
            identity: identity.to_string(),
            role,
        }
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = UserAuthenticationError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    #[tracing::instrument(skip(_payload))]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let current_span = tracing::Span::current();
        tracing::trace!(message = "Authenticating user request");

        let session_storage = req.get_session();

        let req = req.clone();
        Box::pin(async move {
            let dynamodb_client = req
                .app_data::<actix_web::web::Data<crate::authn::AuthDynamoClient>>()
                .ok_or(UserAuthenticationError::AddData)?
                .clone();

            let token: String = session_storage
                .get(crate::config::SESSION_TOKEN)?
                .ok_or(UserAuthenticationError::NoSessionToken)?;

            // tokio 1
            std::thread::spawn(move || {
                use tokio::runtime::Runtime;

                let rt = Runtime::new().unwrap();

                rt.block_on(
                    async move {
                        let session_row =
                            dynamodb_client.get_valid_session_row(token).await.ok()?;
                        let user_row = dynamodb_client
                            .get_user_row(&session_row.username)
                            .await
                            .ok()?;

                        let authenticated_user = AuthenticatedUser {
                            identity: session_row.username,
                            role: user_row.grapl_role,
                        };

                        Some(authenticated_user)
                    }
                    .instrument(current_span),
                )
            })
            .join()
            .unwrap()
            .ok_or(UserAuthenticationError::Opaque)
        })
    }
}
