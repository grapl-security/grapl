// use actix_web::web::{Data, Path};
use actix_web::http::StatusCode;
use actix_web::error::InternalError;
use actix_web::{HttpResponse};
use grapl_router_api::model_plugin_deployer_router::deploy::grapl_model_plugin_deployer;
use std::error::Error;
use actix_web::dev::HttpServiceFactory;

// single public function which returns all of the API request handlers

type StdErr = Box<dyn Error>;

fn to_internal_error(e: StdErr) -> InternalError<StdErr> {
    InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
}

fn to_ok(_: ()) -> HttpResponse {
    HttpResponse::new(StatusCode::OK)
}

pub fn api() -> impl HttpServiceFactory + 'static {
    actix_web::web::scope("/api")
        .service(grapl_model_plugin_deployer)
        // .service(grapl_model_plugin_whatever)
}