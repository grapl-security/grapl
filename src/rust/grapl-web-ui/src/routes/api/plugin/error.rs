use rust_proto::graplinc::grapl::api::protocol::error::GrpcClientError;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum PluginError {
    #[error(transparent)]
    PluginRegistryClient(#[from] GrpcClientError),
    #[error(transparent)]
    Multipart(#[from] actix_multipart::MultipartError),
    #[error("{message}")]
    BadRequest { message: String },
    #[error("unexpected multipart/form-data part: expected '{expected}': found: {found} ")]
    UnexpectedPart {
        expected: &'static str,
        found: String,
    },
    //TODO: bring this back when we can stream the plugin artifact upstream
    // #[error("multipart/form-data chunk too large at {size} bytes: max size: {max_size} bytes")]
    // MultipartChunkTooLarge {
    //     size: usize,
    //     max_size: usize,
    // },
    #[error("unable to parse metadata: {0}")]
    DeserializeMetadata(#[from] serde_json::Error),
    #[error("user request for a resource belonging to another tenant")]
    Unauthorized,
    #[error("gRPC client timeout: {0}")]
    RcpTimeout(#[from] tokio::time::error::Elapsed),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl actix_web::error::ResponseError for PluginError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match *self {
            PluginError::Multipart(_)
            | PluginError::BadRequest { .. }
            | PluginError::UnexpectedPart { .. } => actix_web::HttpResponse::BadRequest().finish(),
            PluginError::Unauthorized => actix_web::HttpResponse::Unauthorized().finish(),
            _ => actix_web::HttpResponse::InternalServerError().finish(),
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            PluginError::Multipart(_)
            | PluginError::BadRequest { .. }
            | PluginError::UnexpectedPart { .. } => actix_web::http::StatusCode::BAD_REQUEST,
            PluginError::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
