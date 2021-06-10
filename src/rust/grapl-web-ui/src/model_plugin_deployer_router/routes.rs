use ::reqwest;
use actix_web::{
    post,
    HttpResponse,
    Responder,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    make_request,
    request_with_body,
};

#[derive(Serialize, Deserialize)]
pub struct DeployRequest {
    name: String,
}

#[derive(thiserror::Error, Debug)]
pub enum PluginError {
    #[error("Request Error")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid schema contents")]
    InvalidSchema,

    #[error("Unable to read schema contents")]
    ReadError,

    #[error("Internal Server Error")]
    ServerError,
}

// accept http requests, translate to grpc requests.

// actix procedural macros that route incoming http requests
#[post("/modelPluginDeployer/deploy")]
pub async fn grapl_model_plugin_deployer(
    body: actix_web::web::Json<DeployRequest>,
) -> impl Responder {
    // CALL MODEL-PLUGIN-DEPlOYER GRPC CLIENT
    let body = body.into_inner();
    let response = request_with_body("deploy", body).await;

    match response {
        Ok(response) => HttpResponse::Ok().json(response),

        Err(PluginError::InvalidSchema) => HttpResponse::BadRequest().finish(),

        Err(PluginError::ReadError) => HttpResponse::Conflict().finish(),

        Err(PluginError::ServerError) => HttpResponse::BadRequest().finish(),

        Err(PluginError::RequestError(_)) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/modelPluginDeployer/deletePlugin")]
pub async fn delete_plugin(body: actix_web::web::Json<DeployRequest>) -> impl Responder {
    let body = body.into_inner();

    let response = request_with_body("delete", body).await;

    match response {
        Ok(response) => HttpResponse::Ok().json(response),

        Err(PluginError::InvalidSchema) => HttpResponse::BadRequest().finish(),

        Err(PluginError::ReadError) => HttpResponse::Conflict().finish(),

        Err(PluginError::ServerError) => HttpResponse::BadRequest().finish(),

        Err(PluginError::RequestError(_)) => HttpResponse::InternalServerError().finish(),
    }
}
// actix procedural macros that route incoming http requests
#[post("/modelPluginDeployer/listPlugins")]
pub async fn list_plugin() -> impl Responder {
    let response = make_request("listPlugins").await;

    match response {
        Ok(response) => HttpResponse::Ok().json(response),

        Err(PluginError::InvalidSchema) => HttpResponse::BadRequest().finish(),

        Err(PluginError::ReadError) => HttpResponse::Conflict().finish(),

        Err(PluginError::ServerError) => HttpResponse::BadRequest().finish(),

        Err(PluginError::RequestError(_)) => HttpResponse::InternalServerError().finish(),
    }
}

// We will make a post request to our new actix server
// This will route us to the appropriate model plugin deployer service.

// we come in on a path. Based on that path, we route the request to the appropriate service.

// X route to the graplModelPluginDeployer
// X set up docker stuff

// setup & write tests with an http client
// use grpc client for model-plugin-deployer

// add more logging
// documentation

// NEXT: Framework for integration
// When will Actix4 Be released? and what is it?
