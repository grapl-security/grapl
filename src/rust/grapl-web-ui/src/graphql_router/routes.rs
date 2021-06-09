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

use crate::graphql_request;

// use crate::graphql_request;

#[derive(Serialize, Deserialize)]
pub struct GraphQLBody {
    body: Vec<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum GraphQLError {
    #[error("RequestError")]
    RequestError(#[from] reqwest::Error),

    #[error("No Content")]
    NoContent,

    #[error("Bad Request")]
    BadRequest,
}

#[post("/graphql")]
pub async fn graphql_router(body: actix_web::web::Json<GraphQLBody>) -> impl Responder {
    let body = body.into_inner();
    let response = graphql_request("graphql", body).await;

    match response {
        Ok(response) => HttpResponse::Ok().json(response),

        Err(GraphQLError::BadRequest) => HttpResponse::BadRequest().finish(),

        Err(GraphQLError::NoContent) => HttpResponse::NoContent().finish(),

        Err(GraphQLError::RequestError(_)) => HttpResponse::InternalServerError().finish(),
    }
}
