// use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::client::Pl
use rust_proto::protocol::error::GrpcClientError;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum WebResponseError {
    #[error(transparent)]
    PluginRegistryClient(#[from] GrpcClientError),
    #[error("unable to parse user's tenant ID from the database: {0}")]
    ParseTenantId(#[from] uuid::Error),
    #[error(transparent)]
    Multipart(#[from] actix_multipart::MultipartError),
    #[error("plugin name not supplied")]
    MissingName,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl actix_web::error::ResponseError for WebResponseError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match *self {
            WebResponseError::Multipart(_) | WebResponseError::MissingName => {
                actix_web::HttpResponse::BadRequest().finish()
            }
            _ => actix_web::HttpResponse::InternalServerError().finish(),
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            WebResponseError::Multipart(_) | WebResponseError::MissingName => {
                actix_web::http::StatusCode::BAD_REQUEST
            }
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
