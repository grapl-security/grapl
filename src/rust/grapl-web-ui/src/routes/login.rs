use actix_web::{
    guard,
    web,
    HttpResponse,
    Responder,
};
use secrecy::ExposeSecret;
use serde::{
    Deserialize,
    Serialize,
};

use crate::authn::{
    AuthenticatedUser,
    Secret,
};

#[derive(Deserialize, Debug)]
struct SignInWithPasswordParameters {
    username: String,
    password: Secret<String>,
}

#[derive(Deserialize, Debug)]
struct SignInWithGoogleParameters {
    token: Secret<String>,
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
            .route(web::post().to(sign_in_with_password))
            .guard(guard::Post())
            .guard(guard::Header("content-type", "application/json")), // .guard(guard::Header("X-Requested-With", "XMLHttpRequest")),
    );
    cfg.service(
        web::resource("/auth/sign_in_with_google")
            .route(web::post().to(sign_in_with_google))
            .guard(guard::Post())
            .guard(guard::Header("content-type", "application/json")), // .guard(guard::Header("X-Requested-With", "XMLHttpRequest")),
    );
}

#[tracing::instrument]
async fn check_login(user: AuthenticatedUser) -> impl Responder {
    tracing::debug!( message = "Checking user session token", username = %user.get_username() );

    HttpResponse::Ok().json(CheckLoginResponse { success: true })
}

#[tracing::instrument(skip(auth_client, data, session), fields(
    username = tracing::field::Empty
))]
async fn sign_in_with_password(
    auth_client: web::Data<crate::authn::WebAuthenticator>,
    session: actix_session::Session,
    data: web::Json<SignInWithPasswordParameters>,
    req: actix_web::HttpRequest,
) -> Result<impl Responder, crate::authn::AuthenticationError> {
    tracing::debug!(message = "processing password authentication request",);

    let username = data.username.to_owned();
    let password = data.password.to_owned();

    let session_token = auth_client
        .sign_in_with_pw(username.clone(), password)
        .await?;

    session.insert(crate::config::SESSION_TOKEN, session_token.expose_secret())?;

    tracing::info!(message = "password authentication success", %username);

    Ok(HttpResponse::Ok().json(CheckLoginResponse { success: true }))
}

#[tracing::instrument(skip(auth_client, data, session), fields(
    username = tracing::field::Empty
))]
async fn sign_in_with_google(
    auth_client: web::Data<crate::authn::WebAuthenticator>,
    session: actix_session::Session,
    data: web::Json<SignInWithGoogleParameters>,
    req: actix_web::HttpRequest,
) -> Result<impl Responder, crate::authn::AuthenticationError> {
    tracing::debug!(message = "processing Sign In With Google authentication request",);

    let session_token = auth_client
        .sign_in_with_google(data.token.to_owned())
        .await?;

    session.insert(crate::config::SESSION_TOKEN, session_token.expose_secret())?;

    tracing::info!(message = "user completed Sign In With Google");

    Ok(HttpResponse::Ok().json(CheckLoginResponse { success: true }))
}
