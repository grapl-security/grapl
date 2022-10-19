use rust_proto::protocol::error::GrpcClientError;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum IngressError {
    #[error(transparent)]
    PluginRegistryClient(#[from] GrpcClientError),
    #[error(transparent)]
    Payload(#[from] actix_web::error::PayloadError),
    #[error("gRPC client timeout: {0}")]
    RcpTimeout(#[from] tokio::time::error::Elapsed),
}

impl actix_web::error::ResponseError for IngressError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match *self {
            IngressError::Payload(_) => actix_web::HttpResponse::BadRequest().finish(),
            _ => actix_web::HttpResponse::InternalServerError().finish(),
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            IngressError::Payload(_) => actix_web::http::StatusCode::BAD_REQUEST,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
