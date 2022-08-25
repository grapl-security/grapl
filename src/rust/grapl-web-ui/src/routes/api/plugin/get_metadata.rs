use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use std::sync::Mutex;
use uuid::Uuid;

use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetPluginRequest,
    PluginRegistryServiceClient,
};

#[derive(serde::Deserialize)]
pub(super) struct GetPluginMetadataParameters {
    plugin_id: Uuid,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GetPluginMetadata<'a> {
    pub plugin_id: Uuid,
    pub display_name: &'a str,
    pub plugin_type: &'a str,
    pub event_source_id: Option<uuid::Uuid>,
}

#[tracing::instrument(skip(plugin_registry_client, user, data), fields(
    username = tracing::field::Empty
))]
pub(super) async fn get_metadata(
    plugin_registry_client: web::Data<Mutex<PluginRegistryServiceClient>>,
    user: crate::authn::AuthenticatedUser,
    data: web::Query<GetPluginMetadataParameters>,
) -> Result<impl Responder, GetPluginMetadataResponseError> {
    let tenant_id = user.get_organization_id().to_owned();

    let request = GetPluginRequest::new(data.plugin_id, tenant_id);

    let mut plugin_registry_client = plugin_registry_client.lock().unwrap();
    let plugin_registry_response = plugin_registry_client.get_plugin(request).await?;

    let plugin_metadata = plugin_registry_response.plugin_metadata();

    let web_response = GetPluginMetadata {
        plugin_id: plugin_registry_response.plugin_id(),
        display_name: plugin_metadata.display_name(),
        plugin_type: plugin_metadata.plugin_type().type_name(),
        event_source_id: plugin_metadata.event_source_id(),
    };

    Ok(HttpResponse::Ok().json(web_response))
}





use rust_proto::protocol::error::GrpcClientError;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum GetPluginMetadataResponseError {
    #[error(transparent)]
    PluginRegistryClient(#[from] GrpcClientError),
    #[error("unable to parse user's tenant ID from the database: {0}")]
    ParseTenantId(#[from] uuid::Error),
    #[error(transparent)]
    Multipart(#[from] actix_multipart::MultipartError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl actix_web::error::ResponseError for GetPluginMetadataResponseError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match *self {
            GetPluginMetadataResponseError::Multipart(_) => {
                actix_web::HttpResponse::BadRequest().finish()
            }
            _ => actix_web::HttpResponse::InternalServerError().finish(),
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            GetPluginMetadataResponseError::Multipart(_) => {
                actix_web::http::StatusCode::BAD_REQUEST
            }
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
