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

// impl FromRequest for AuthenticatedUser {
//     type Error = actix_web::HttpResponse;
//     type Future = Ready<Result<Self, Self::Error>>;
//     type Config = ();

//     fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
//         let session_storage = req.get_session();

//         // let try_get_user = || -> async move {
//         //     let dynamodb_client = req.app_data::<actix_web::web::Data<crate::authn::AuthDynamoClient>>().unwrap();
//         //     let session_storage = req.get_session();

//         // };

//         // let try_get_user = || -> Option<AuthenticatedUser> {
//         //     let dynamodb_client = req.app_data::<actix_web::web::Data<crate::authn::AuthDynamoClient>>()?;
//         //     let session_storage = req.get_session();
//         //     let session_token: String = session_storage
//         //         .get(SESSION_TOKEN)
//         //         .map_err(|_| WebSecError::MissingSession)?
//         //         .ok_or(WebSecError::MissingSession)?;

//         //     let session_row = dynamodb_client.get_valid_session_row(session_token).await?;
//         //     let user_row = dynamodb_client.get_user_row(&session_row.username).await?;

//         //     let authenticated_user = AuthenticatedUser {
//         //         identity: session_row.username,
//         //         role: user_row.role,
//         //     };

//         //     Some(authenticated_user)

//         //     // let users = req.app_data::<actix_web::web::Data<Users>>()?;
//         //     // let session = req.get_session();
//         //     // let username_from_cookie = session.get::<String>(KEY).ok()??;
//         //     // let user = users.get_from_email(username_from_cookie.as_str())?;
//         //     // Some(AuthenticatedUser(user.clone()))
//         // };

//         let err_redirect = || {
//             actix_web::HttpResponse::Found()
//                 .header(actix_web::http::header::LOCATION, "/login")
//                 .finish()
//                 .into_body()
//         };

//         futures_util::future::ready(try_get_user().ok_or_else(err_redirect))
//     }
// }

// impl FromRequest for AuthenticatedUser {
//     // type Error = WebSecError;
//     type Error = actix_web::Error;
//     // type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
//     type Future = Ready<Result<Self, Self::Error>>;
//     type Config = ();

//     fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
//         // use crate::authn::dynamodb;
//         use actix_session::UserSession;

//         let session_storage = req.get_session();

//         // let dynamodb_client = req
//         //     .app_data::<actix_web::web::Data<crate::authn::AuthDynamoClient>>()
//         //     .ok_or_else(internal_server_error)?;

//         let dynamodb_client = req
//             .app_data::<actix_web::web::Data<crate::authn::AuthDynamoClient>>();

//         let a = match dynamodb_client {
//             Some(client) => {
//                 ok(AuthenticatedUser {
//                     identity: "asdf".to_owned(),
//                     role: GraplRole::Administrator,
//                 })
//             },
//             None => {
//                 err(actix_web::error::ErrorBadRequest("no luck"))
//             }
//         };

//         let b = a?;

//         let attempt_authentication = Ok(AuthenticatedUser {
//             identity: "asdf".to_owned(),
//             role: GraplRole::Administrator,
//         });

//         // let attempt_authentication = async move {
//         //     let session_token: String = session_storage
//         //         .get(SESSION_TOKEN)
//         //         .map_err(|_| WebSecError::MissingSession)?
//         //         .ok_or(WebSecError::MissingSession)?;

//         //     let session_row = dynamodb_client.get_valid_session_row(session_token).await?;
//         //     let user_row = dynamodb_client.get_user_row(&session_row.username).await?;

//         //     let authenticated_user = AuthenticatedUser {
//         //         identity: session_row.username,
//         //         role: user_row.role,
//         //     };

//         //     Ok(authenticated_user)
//         // };

//         futures_util::future::ready(attempt_authentication)
//         // Box::pin(attempt_authentication)
//     }
// }
