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
use tracing::Instrument;

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
            .route(web::post().to(post_compat))
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
async fn post_compat(
    db_client: web::Data<crate::authn::AuthDynamoClient>,
    session: actix_session::Session,
    data: web::Json<LoginParameters>,
) -> impl Responder {
    let current_span = tracing::Span::current();
    current_span.record("username", &data.username.as_str());

    tracing::debug!(message = "Processing authentication request.",);

    let username = data.username.clone();
    let password = Password::from(data.password.clone());

    // This is only necessary until we move to actix_web 4, which uses a tokio 1 runtime.
    std::thread::spawn(move || {
        use tokio::runtime::Runtime;

        let rt = Runtime::new().unwrap();

        rt.block_on(
            async move { db_client.sign_in(username.as_str(), &password).await }
                .instrument(current_span),
        )
    })
    .join()
    .unwrap()
    .map_or_else(
        |e| {
            match e {
                // incorrect password
                AuthDynamoClientError::PasswordVerification(
                    argon2::password_hash::Error::Password,
                )
                | AuthDynamoClientError::UserRecordNotFound(_) => {
                    tracing::info!( message = %e );
                    HttpResponse::Unauthorized().body(format!("{}", e))
                }
                _ => {
                    tracing::error!( error = %e );
                    HttpResponse::InternalServerError().finish()
                }
            }
        },
        |web_session| {
            session
                .set(crate::config::SESSION_TOKEN, web_session.get_token())
                .map_or_else(
                    |e| {
                        tracing::error!( message = "Unable to set session data.", error = %e );
                        HttpResponse::InternalServerError().finish()
                    },
                    |_| HttpResponse::Ok().json(CheckLoginResponse { success: true }),
                )
        },
    )
}

// TODO(inickles): Keep this for now. It will be useful for the move to actix_web 4,
// which uses tokio 1.
//
// #[tracing::instrument(skip(db_client, data, session), fields(
//     username = tracing::field::Empty
// ))]
// async fn post(
//     db_client: web::Data<crate::authn::AuthDynamoClient>,
//     session: actix_session::Session,
//     data: web::Json<LoginParameters>,
// ) -> impl Responder {
//     let current_span = tracing::Span::current();
//     current_span.record("username", &data.username.as_str());

//     tracing::debug!(message = "Processing authentication request.",);

//     let err_resp = |msg: String| HttpResponse::Unauthorized().body(msg);

//     let username = data.username.clone();
//     let password = Password::from(data.password.clone());

//     db_client
//         .sign_in(username.as_str(), &password)
//         .await
//         .map_or_else(
//             |e| err_resp(format!("{:?}", e)),
//             |(session_token, _)| {
//                 session
//                     .set(crate::config::SESSION_COOKIE_NAME, session_token)
//                     .map_or_else(
//                         |e| err_resp(format!("Unable to set session data: {}", e)),
//                         |_| {
//                             HttpResponse::Ok()
//                                 .body(format!("Sign in success for: {}", username.as_str()))
//                         },
//                     )
//             },
//         )
// }
