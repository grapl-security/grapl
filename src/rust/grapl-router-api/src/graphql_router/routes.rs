use crate::{graphql_request_with_body};
use actix_web::{post, get, Error, HttpResponse, Responder};
use::reqwest;
use actix_web::body::Body;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct GraphQLRes {
    body: Vec<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum GraphQLError {
    #[error("RequestError")]
    RequestError(#[from] reqwest::Error),

    #[error("Internal Server Error")]
    ServerError,

    #[error("No Content")]
    NoContent,
}

#[post("/graphql")]
pub async fn graphql(body: actix_web::web::Json<GraphQLRes>) -> impl Responder {
    let body = body.into_inner();
    let response = request_with_body("graphql", body)
        .await;

    match response{
        Ok(response) => HttpResponse::Ok().json(response),

        Err(GraphQLError::BadRequest) => {
            HttpResponse::BadRequest()
                .finish()
        }

        Err(GraphQLError::NoContent) => {
            HttpResponse::NoContent()
                .finish()
        }


        Err(GraphQLError::RequestError(_)) =>  HttpResponse::InternalServerError().finish(),

    }
}
