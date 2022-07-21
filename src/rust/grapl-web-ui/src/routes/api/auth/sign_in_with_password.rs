use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use secrecy::ExposeSecret;

use crate::authn::Secret;

#[derive(serde::Deserialize)]
pub(super) struct SignInWithPasswordParameters {
    username: String,
    password: Secret<String>,
}

#[derive(serde::Serialize)]
struct SignInWithPasswordResponse {
    success: bool,
}

#[tracing::instrument(skip(auth_client, data, session), fields(
    username = tracing::field::Empty
))]
pub(super) async fn sign_in_with_password(
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

    Ok(HttpResponse::Ok().json(SignInWithPasswordResponse { success: true }))
}
