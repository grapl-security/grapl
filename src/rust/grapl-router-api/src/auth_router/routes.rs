use crate::{login_request_with_body};
use actix_web::{post, get, Error, HttpResponse, Responder};
use::reqwest;
use actix_web::body::Body;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct LoginBody {
    username: String,
    password: String,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Request Error")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid creds")]
    InvalidCreds,

}

#[get("/login")]
pub async fn grapl_login(body: actix_web::web::Json<LoginBody>) -> impl Responder {
    let body = body.into_inner();
    let response = login_request_with_body("login", body)
        .await;

    match response {
        Ok(response) => HttpResponse::Ok().json(response),

        Err(AuthError::InvalidCreds) => {
            HttpResponse::Forbidden()
                .finish()
        }

        Err(AuthError::RequestError(_)) =>  HttpResponse::InternalServerError().finish(),
    }
}
