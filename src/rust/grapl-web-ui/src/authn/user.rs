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
    #[error("unable to validate user session")]
    SessionValidation(#[source] AuthDynamoClientError),
    #[error("unable to get user data from validated session")]
    UserNotFound(#[source] AuthDynamoClientError),
    #[error("unable to access session storage")]
    SessionStorage(#[source] actix_web::Error),
    #[error("user is not authenticated")]
    Unauthenticated,
    #[error("unable to get database client from Actix app data")]
    DynamoClientUnavailable,
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

    #[allow(dead_code)]
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
            role,
        }
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = UserAuthenticationError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    #[tracing::instrument(skip(_payload))]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        tracing::trace!(message = "Authenticating user request");

        let session_storage = req.get_session();

        //TODO(inickles): stop hitting the database for each request
        let req = req.clone();
        Box::pin(async move {
            let dynamodb_client = req
                .app_data::<actix_web::web::Data<crate::authn::AuthDynamoClient>>()
                .ok_or(UserAuthenticationError::DynamoClientUnavailable)?;

            let token = session_storage
                .get::<String>(crate::config::SESSION_TOKEN)
                .map_err(|e| UserAuthenticationError::SessionStorage(e.into()))?
                .ok_or(UserAuthenticationError::Unauthenticated)?;

            let session_row = dynamodb_client
                .get_valid_session_row(token)
                .await
                .map_err(|e| UserAuthenticationError::SessionValidation(e.into()))?;

            let user_row = dynamodb_client
                .get_user_row(&session_row.username)
                .await
                .map_err(|e| UserAuthenticationError::UserNotFound(e.into()))?;

            let authenticated_user = AuthenticatedUser {
                identity: session_row.username,
                role: user_row.grapl_role,
            };

            Ok(authenticated_user)
        })
    }
}
