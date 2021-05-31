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



// We will make a post request to our new actix server
// This will route us to the appropriate model plugin deployer service.

// we come in on a path. Based on that path, we route the request to the appropriate service.

// X route to the graplModelPluginDeployer
// setup & write tests with an http client
// use grcp client for model-plugin-deployer
// X set up docker stuff

// every service can have a directory
// every route in service can have a file
