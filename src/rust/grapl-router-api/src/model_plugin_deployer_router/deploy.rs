use crate::make_request;
use actix_web::{post, Responder, HttpResponse};


// actix procedural macros that route incoming http requests
#[post("/modelPluginDeployer/deploy")]
pub async fn grapl_model_plugin_deployer(req_body: String) -> impl Responder { // IDK what this body is yet - will change types
    let body = HttpResponse::Ok().body(req_body);
    // CALL MODEL-PLUGIN-DEPOYER GRPC CLIENT
    make_request("/modelPluginDeployer/deploy", body);



    // We will make a post request to our new actix server
    // This will route us to the appropriate model plugin deployer service.

    // we come in on a path. Based on that path, we route the request to the appropriate service.

    // route to the graplModelPluginDeployer
    // setup & write tests with an http client
    // use grcp client for model-plugin-deployer
    // X set up docker stuff
    // every service can have a directory
    // every route in service can have a file
    // setup & write tests with an http client

}
