#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct CheckLoginResponse {
    success: bool,
}

#[tracing::instrument]
pub(super) async fn check_login(
    user: crate::authn::AuthenticatedUser,
) -> impl actix_web::Responder {
    tracing::debug!( message = "Checking user session token", username = %user.get_username() );

    actix_web::HttpResponse::Ok().json(CheckLoginResponse { success: true })
}
