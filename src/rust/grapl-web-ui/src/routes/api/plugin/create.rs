use actix_multipart::Multipart;
use actix_web::web::{self,};
use futures::{
    StreamExt,
    TryStreamExt,
};
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    PluginMetadata,
    PluginRegistryClient,
    PluginType,
};

use super::PluginError;

// TODO: Reintroduce this when we can stream plugin artifact upstream
// const MAX_SIZE: usize = 256 * 1024; // max payload size is 256k

#[derive(serde::Deserialize)]
pub(super) struct CreateParametersMetadata {
    plugin_name: String,
    #[serde(deserialize_with = "deserialize_plugin_type")]
    plugin_type: PluginType,
    event_source_id: Option<uuid::Uuid>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct CreateResponse {
    pub plugin_id: uuid::Uuid,
}

/// Upload a plugin.
///
/// Request should look something along the lines of the follwing:
///
/// // Content-Type: multipart/form-data; boundary=------------------------2eb075b38ea8b578
///
/// // --------------------------2eb075b38ea8b578
/// // Content-Disposition: form-data; name="metadata"
///
/// // {"plugin_name": "Asdf Plugin", "plugin_type": "generator", "event_source_id": "00000000-0000-0000-0000-000000000000"}
/// // --------------------------2eb075b38ea8b578
/// // Content-Disposition: form-data; name="upload"; filename="id"
/// // Content-Type: application/octet-stream
/// //
/// // <bytes>
#[tracing::instrument(skip(plugin_registry_client, payload))]
pub(super) async fn create(
    plugin_registry_client: web::Data<PluginRegistryClient>,
    user: crate::authn::AuthenticatedUser,
    mut payload: Multipart,
) -> Result<impl actix_web::Responder, PluginError> {
    let tenant_id = user.get_organization_id();

    let metadata = get_metadata(&mut payload).await?;

    let plugin_artifact = get_plugin_artifact(&mut payload).await?;
    let plugin_artifact_stream = futures::stream::once(async move { plugin_artifact });

    let plugin_metadata = PluginMetadata::new(
        tenant_id.to_owned(),
        metadata.plugin_name,
        metadata.plugin_type,
        metadata.event_source_id,
    );

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();
    let response = plugin_registry_client
        .create_plugin(
            std::time::Duration::from_secs(5),
            plugin_metadata,
            plugin_artifact_stream,
        )
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let plugin_id = response.plugin_id();

    Ok(actix_web::HttpResponse::Ok().json(CreateResponse { plugin_id }))
}

async fn get_metadata(payload: &mut Multipart) -> Result<CreateParametersMetadata, PluginError> {
    if let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition
            .get_name()
            .ok_or(PluginError::BadRequest {
                message: "multipart/form-data name missing".to_string(),
            })?;

        let metadata_field_name = "metadata";

        if field_name != metadata_field_name {
            return Err(PluginError::UnexpectedPart {
                expected: metadata_field_name,
                found: field_name.to_string(),
            });
        }

        // Get form body
        let mut body = web::BytesMut::new();
        if let Some(chunk) = field.next().await {
            let chunk = chunk?;

            // TODO: Reintroduce this when we can stream plugin artifact upstream
            // // limit max size of in-memory payload
            // if (body.len() + chunk.len()) > MAX_SIZE {
            //     return Err(WebResponseError::MultipartChunkTooLarge {
            //         size: chunk.len(),
            //         max_size: MAX_SIZE,
            //     })
            // }

            body.extend_from_slice(&chunk);
        }

        let metadata = serde_json::from_slice::<CreateParametersMetadata>(&body)?;

        return Ok(metadata);
    }

    Err(PluginError::BadRequest {
        message: "missing form data for plugin/create metadata".to_string(),
    })
}

async fn get_plugin_artifact(payload: &mut Multipart) -> Result<web::Bytes, PluginError> {
    if let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition
            .get_name()
            .ok_or(PluginError::BadRequest {
                message: "multipart/form-data name missing".to_string(),
            })?;

        let metadata_field_name = "plugin_artifact";

        if field_name != metadata_field_name {
            return Err(PluginError::UnexpectedPart {
                expected: metadata_field_name,
                found: field_name.to_string(),
            });
        }

        // Get form body
        let mut body = web::BytesMut::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk?;
            body.extend_from_slice(&chunk);
        }

        return Ok(body.into());
    }

    Err(PluginError::BadRequest {
        message: "missing form data for plugin/create plugin artifact".to_string(),
    })
}

// It'd be greate if we didn't have to do this and PluginType just derived serde::Deserialize itself
static PLUGIN_TYPE_EXPECTED: &'static [&'static str] = &["generator", "analyzer"];

fn deserialize_plugin_type<'de, D>(deserializer: D) -> Result<PluginType, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let value: &str = serde::de::Deserialize::deserialize(deserializer)?;
    PluginType::try_from(value)
        .map_err(|_| serde::de::Error::unknown_variant(value, PLUGIN_TYPE_EXPECTED))
}
