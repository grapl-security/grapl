use actix_web::{
    guard,
    web,
    HttpResponse,
    Responder,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::authn::{
    AuthDynamoClientError,
    AuthenticatedUser,
    Password,
};

#[derive(Deserialize, Debug)]
struct LoginParameters {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CheckLoginResponse {
    success: bool,
}

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/auth/checkLogin")
            .route(web::post().to(check_login))
            .guard(guard::Post())
            .guard(guard::Header("content-type", "application/json")), // .guard(guard::Header("X-Requested-With", "XMLHttpRequest")),
    );
    cfg.service(
        web::resource("/auth/login")
            .route(web::post().to(post))
            .guard(guard::Post())
            .guard(guard::Header("content-type", "application/json")), // .guard(guard::Header("X-Requested-With", "XMLHttpRequest")),
    );
}

#[tracing::instrument]
async fn check_login(user: AuthenticatedUser) -> impl Responder {
    tracing::debug!( message = "Checking user session token", identity = %user.get_identity() );

    HttpResponse::Ok().json(CheckLoginResponse { success: true })
}

#[tracing::instrument(skip(db_client, data, session), fields(
    username = tracing::field::Empty
))]
async fn post(
    db_client: web::Data<crate::authn::AuthDynamoClient>,
    session: actix_session::Session,
    data: web::Json<LoginParameters>,
) -> impl Responder {
    let current_span = tracing::Span::current();
    current_span.record("username", &data.username.as_str());

    tracing::debug!(message = "processing authentication request",);

    let username = data.username.as_str();
    let password = Password::from(data.password.clone());

    db_client.sign_in(username, &password).await.map_or_else(
        |error| {
            match error {
                // incorrect password
                AuthDynamoClientError::PasswordVerification(
                    argon2::password_hash::Error::Password,
                )
                | AuthDynamoClientError::UserRecordNotFound(_) => {
                    tracing::info!( %error );
                    HttpResponse::Unauthorized().finish()
                }
                _ => {
                    tracing::error!( %error );
                    HttpResponse::InternalServerError().finish()
                }
            }
        },
        |web_session| {
            session
                .insert(crate::config::SESSION_TOKEN, web_session.get_token())
                .map_or_else(
                    |error| {
                        tracing::error!( message = "unable to set session data", %error );
                        HttpResponse::InternalServerError().finish()
                    },
                    |_| HttpResponse::Ok().json(CheckLoginResponse { success: true }),
                )
        },
    )
}
