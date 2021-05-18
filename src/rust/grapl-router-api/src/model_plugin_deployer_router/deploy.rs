use actix_web::{post, Responder};
// use actix_web::error::InternalError;
// use actix_web::http::StatusCode;

use serde;

#[derive(serde::Serialize)]
pub struct Response {
    pub res: String
}

// actix procedural macros that route incoming http requests
#[post("/modelPluginDeployer/deploy")]
pub async fn grapl_model_plugin_deployer() -> impl Responder {
    //curl("http:colinapi.com")
    //.await

    // CALL MODEL-PLUGIN-DEPOYER GRPC CLIENT

    // we come in on a path. Based on that path, we route the request to the appropriate service.

    // route to the graplModelPluginDeployer
    // use grcp client for model-plugin-deployer
    // setup & write tests with an http client
    // use grcp client for model-plugin-deployer
    // X set up docker stuff
    // every service can have a directory
    // every route in service can have a file
    // route to the graplModelPluginDeployer
    // setup & write tests with an http client

    ""
}
