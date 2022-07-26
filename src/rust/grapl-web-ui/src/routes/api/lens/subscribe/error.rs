use rust_proto::graplinc::grapl::api::lens_subscription_service::v1beta1::client::LensSubscriptionClientError;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum WebResponseError {
    #[error(transparent)]
    LensSubscriptionClient(#[from] LensSubscriptionClientError),
    #[error("unable to JSON serialize response from lens subscription client: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Unauthorized(#[from] crate::authn::AuthenticationError),
    #[error("lens ID cannot be zero")]
    LensUidZero,
    #[error("tenant ID from user session `{tenant_id}` is not a UUID: {source}")]
    InavlidTenantID {
        tenant_id: String,
        source: uuid::Error,
    },
}

impl actix_web::error::ResponseError for WebResponseError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match *self {
            WebResponseError::LensUidZero => {
                actix_web::HttpResponse::UnprocessableEntity().finish()
            }
            _ => actix_web::HttpResponse::InternalServerError().finish(),
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            WebResponseError::LensUidZero => actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
