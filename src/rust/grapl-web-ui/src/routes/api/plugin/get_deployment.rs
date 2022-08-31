use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetPluginDeploymentRequest,
    PluginDeploymentStatus,
    PluginRegistryServiceClient,
};

use super::PluginError;

#[derive(serde::Deserialize)]
pub(super) struct GetDeploymentParameters {
    plugin_id: uuid::Uuid,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PluginDeploymentResponse {
    pub plugin_id: uuid::Uuid,
    pub timestamp: std::time::SystemTime,
    #[serde(serialize_with = "serialize_deployment_status")]
    #[serde(deserialize_with = "deserialize_deployment_status")]
    pub status: PluginDeploymentStatus,
    pub deployed: bool,
}

#[tracing::instrument(skip(plugin_registry_client, data))]
pub(super) async fn get_deployment(
    plugin_registry_client: web::Data<PluginRegistryServiceClient>,
    user: crate::authn::AuthenticatedUser,
    data: web::Query<GetDeploymentParameters>,
) -> Result<impl Responder, PluginError> {
    let requested_plugin_id = data.plugin_id;

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();

    super::verify_plugin_ownership(&mut plugin_registry_client, &user, requested_plugin_id).await?;

    let request = GetPluginDeploymentRequest::new(requested_plugin_id);
    let response = plugin_registry_client
        .get_plugin_deployment(request)
        .await?
        .plugin_deployment();

    let web_response = PluginDeploymentResponse {
        plugin_id: response.plugin_id(),
        timestamp: response.timestamp(),
        status: response.status(),
        deployed: response.deployed(),
    };

    Ok(HttpResponse::Ok().json(web_response))
}

//TODO: It'd be great if rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginDeploymentStatus
// implemented Display, or serde::Serialize
fn serialize_deployment_status<S>(
    health_status: &PluginDeploymentStatus,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    match *health_status {
        PluginDeploymentStatus::Unspecified => {
            s.serialize_unit_variant("PluginDeploymentStatus", 0, "unspecified")
        }
        PluginDeploymentStatus::Success => {
            s.serialize_unit_variant("PluginDeploymentStatus", 0, "success")
        }
        PluginDeploymentStatus::Fail => {
            s.serialize_unit_variant("PluginDeploymentStatus", 0, "fail")
        }
    }
}

static DEPLOYMENT_STATUS_EXPECTED: &'static [&'static str] = &["unspecified", "success", "fail"];
fn deserialize_deployment_status<'de, D>(
    deserializer: D,
) -> Result<PluginDeploymentStatus, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let value: &str = serde::de::Deserialize::deserialize(deserializer)?;
    match value {
        "unspecified" => Ok(PluginDeploymentStatus::Unspecified),
        "success" => Ok(PluginDeploymentStatus::Success),
        "fail" => Ok(PluginDeploymentStatus::Fail),
        value => Err(serde::de::Error::unknown_variant(
            value,
            DEPLOYMENT_STATUS_EXPECTED,
        )),
    }
}
