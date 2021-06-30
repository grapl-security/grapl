use std::fmt::{
    Display,
    Formatter,
};

use actix_web::{
    http::StatusCode,
    ResponseError,
};
use rusoto_core::RusotoError;
use rusoto_dynamodb::GetItemError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSecError {
    DatabaseError(#[from] RusotoError<GetItemError>),
    MissingSession,
    InvalidSession,
    AuthenticationFailure,
    ArgonError(argon2::Error),
    PasswordHashingError(argon2::password_hash::Error),
    SessionCreationError,
    Unauthorized,
}

impl ResponseError for WebSecError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

impl Display for WebSecError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            WebSecError::MissingSession
            | WebSecError::InvalidSession
            | WebSecError::AuthenticationFailure => formatter.write_str("Unauthenticated"),
            _ => formatter.write_str("Unauthorized"),
        }
    }
}
