pub type Result<T> = std::result::Result<T, AuthenticationError>;
use jsonwebtoken_google::ParserError as GoogleTokenError;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum AuthenticationError {
    #[error(transparent)]
    DbClient(#[from] super::dynamodb_client::AuthDynamoClientError),
    #[error("user not found: `{0}`")]
    UserNotFound(String),
    #[error("unable to verify Sign In With Google token: {0}")]
    GoogleTokenVerification(#[from] GoogleTokenError),
    #[error(transparent)]
    Actix(#[from] actix_web::Error),
    #[error("unable to create password hasher: {0}")]
    PasswordHasher(#[from] argon2::Error),
    #[error("unable to deserialize password hash from database: {source}")]
    DeserializePasswordFromDatabase {
        source: argon2::password_hash::Error,
    },
    #[error("invalid password for user: '{username}': {source}")]
    InvalidPassword {
        username: String,
        source: argon2::password_hash::Error,
    },
    #[error("session token not found in database")]
    SessionTokenNotFound,
    #[error("session expired")]
    SessionExpired,
    #[error("error parsing tenant_id '{input}': {source}")]
    ParseOrgId {
        input: String,
        source: uuid::Error,
    }
}

impl actix_web::error::ResponseError for AuthenticationError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match *self {
            AuthenticationError::Actix(_)
            | AuthenticationError::GoogleTokenVerification(GoogleTokenError::KeyProvider(_))
            | AuthenticationError::DeserializePasswordFromDatabase { .. }
            | AuthenticationError::PasswordHasher(_) => {
                actix_web::HttpResponse::InternalServerError().finish()
            }
            _ => actix_web::HttpResponse::Unauthorized().finish(),
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            AuthenticationError::Actix(_)
            | AuthenticationError::GoogleTokenVerification(GoogleTokenError::KeyProvider(_))
            | AuthenticationError::DeserializePasswordFromDatabase { .. }
            | AuthenticationError::PasswordHasher(_) => {
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            _ => actix_web::http::StatusCode::UNAUTHORIZED,
        }
    }
}
