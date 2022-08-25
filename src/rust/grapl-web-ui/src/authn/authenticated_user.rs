use std::pin::Pin;

use actix_session::UserSession;
use actix_web::{
    dev::Payload,
    http::StatusCode,
    FromRequest,
    HttpRequest,
    ResponseError,
};
// use futures_util::future::Future;
use futures::future::Future;

use super::{
    GraplRole,
    Secret,
};

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum SessionValidationError {
    #[error("session token not present")]
    MissingSessionTokenFromRequest,
    #[error("unable to access session storage: {0}")]
    SessionStorage(#[from] actix_web::Error),
    #[error("unable to extract authentication client from Actix app data")]
    ClientUnavailable,
    #[error("unable to validate session: {0}")]
    Validation(#[from] super::AuthenticationError),
}

impl ResponseError for SessionValidationError {
    fn status_code(&self) -> StatusCode {
        match self {
            SessionValidationError::ClientUnavailable
            | SessionValidationError::SessionStorage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::UNAUTHORIZED,
        }
    }
}

/// Represents a user that has been authenticated.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    username: String,
    role: GraplRole,
    organization_id: uuid::Uuid,
}

impl AuthenticatedUser {
    pub(super) fn new(username: String, role: GraplRole, organization_id: uuid::Uuid) -> Self {
        Self {
            username,
            role,
            organization_id,
        }
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_role(&self) -> &GraplRole {
        &self.role
    }

    pub fn get_organization_id(&self) -> &uuid::Uuid {
        &self.organization_id
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = SessionValidationError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    // Do not instrument `err` here. Errors from the `auth` module will have already been logged
    // and some error type variants shouldn't be logged with Level::ERROR, such as
    // UserAuthenticationError::Unauthenticated, which we expect in normal operation.
    #[tracing::instrument(skip(_payload))]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        tracing::trace!("Authenticating user request");

        let session_storage = req.get_session();

        //TODO(inickles): stop hitting the database for each request
        let req = req.clone();
        Box::pin(async move {
            let auth_client = req
                .app_data::<actix_web::web::Data<crate::authn::WebAuthenticator>>()
                .ok_or(SessionValidationError::ClientUnavailable)?;

            let session_token_from_request = session_storage
                .get::<Secret<String>>(crate::config::SESSION_TOKEN)?
                .ok_or(SessionValidationError::MissingSessionTokenFromRequest)?;

            let user = auth_client
                .validate_session_token(session_token_from_request)
                .await?;

            tracing::debug!(
                message = "validated user session",
                username = user.get_username(),
                role = user.get_role().to_string(),
                organization_id =% user.get_organization_id()
            );

            Ok(user)
        })
    }
}
