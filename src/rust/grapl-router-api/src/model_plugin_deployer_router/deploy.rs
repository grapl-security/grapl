use crate::make_request;
use actix_web::{post, Error, HttpResponse, Responder};
use::reqwest;
use actix_web::body::Body;
use serde::{Serialize, Deserialize};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum DeployError {
    #[error("Source contains no data")]
    EmptySource,

    /// Represents a failure to read from input.
    #[error("Read error")]
    ReadError { source: std::io::Error },

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize)]
pub struct DeployRequest{
    name: String,
}

// actix procedural macros that route incoming http requests
#[post("/modelPluginDeployer/deploy")]
pub async fn grapl_model_plugin_deployer(body: actix_web::web::Json<DeployRequest>) -> impl Responder {
    // CALL MODEL-PLUGIN-DEPlOYER GRPC CLIENT
    let body = body.into_inner();
    let response = make_request("deploy", body)
        .await;

    match response{
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            if e == 0 {
                return Err(DeployError::EmptySource);
            }

            if e == "read error"{
                Err(DeployError::ReadError);
            } else {
                return Err(DeployError::IOError);
            }


        }
    }
}


// We will make a post request to our new actix server
// This will route us to the appropriate model plugin deployer service.

// we come in on a path. Based on that path, we route the request to the appropriate service.

// route to the graplModelPluginDeployer
// setup & write tests with an http client
// use grcp client for model-plugin-deployer
// X set up docker stuff

// every service can have a directory
// every route in service can have a file
