use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use secrecy::ExposeSecret;

use crate::authn::Secret;

#[derive(serde::Deserialize)]
pub(super) struct SignInWithGoogleParameters {
    token: Secret<String>,
}

#[derive(serde::Serialize)]
struct SignInWithGoogleResponse {
    success: bool,
}

#[tracing::instrument(skip(auth_client, data, session), fields(
    username = tracing::field::Empty
))]
pub(super) async fn sign_in_with_google(
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

    Ok(HttpResponse::Ok().json(SignInWithGoogleResponse { success: true }))
}
