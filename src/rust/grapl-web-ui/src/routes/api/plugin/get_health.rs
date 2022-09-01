use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetPluginHealthRequest,
    PluginHealthStatus,
    PluginRegistryServiceClient,
};
use uuid::Uuid;

use super::PluginError;

#[derive(serde::Deserialize)]
pub(super) struct GetPluginHealthParameters {
    plugin_id: Uuid,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GetPluginHealthResponse {
    #[serde(serialize_with = "serialize_health_status")]
    #[serde(deserialize_with = "deserialize_health_status")]
    pub health_status: PluginHealthStatus,
}

#[tracing::instrument(skip(plugin_registry_client, data))]
pub(super) async fn get_health(
    plugin_registry_client: web::Data<PluginRegistryServiceClient>,
    user: crate::authn::AuthenticatedUser,
    data: web::Query<GetPluginHealthParameters>,
) -> Result<impl Responder, PluginError> {
    let requested_plugin_id = data.plugin_id;

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();

    super::verify_plugin_ownership(&mut plugin_registry_client, &user, requested_plugin_id).await?;

    let request = GetPluginHealthRequest::new(data.plugin_id);

    tracing::debug!(message = "getting plugin metadata", ?request);

    let plugin_registry_response = plugin_registry_client
        .get_plugin_health(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let web_response = GetPluginHealthResponse {
        health_status: plugin_registry_response.health_status(),
    };

    Ok(HttpResponse::Ok().json(web_response))
}

//TODO: It'd be great if rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginHealthStatus
// implemented Display, or serde::Serialize
fn serialize_health_status<S>(health_status: &PluginHealthStatus, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    match *health_status {
        PluginHealthStatus::NotDeployed => {
            s.serialize_unit_variant("PluginHealthStatus", 0, "not_deployed")
        }
        PluginHealthStatus::Pending => s.serialize_unit_variant("PluginHealthStatus", 1, "pending"),
        PluginHealthStatus::Running => s.serialize_unit_variant("PluginHealthStatus", 2, "running"),
        PluginHealthStatus::Dead => s.serialize_unit_variant("PluginHealthStatus", 3, "dead"),
    }
}

static PLUGIN_HEALTH_EXPECTED: &'static [&'static str] =
    &["not_deployed", "pending", "running", "dead"];
fn deserialize_health_status<'de, D>(deserializer: D) -> Result<PluginHealthStatus, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let value: &str = serde::de::Deserialize::deserialize(deserializer)?;
    match value {
        "not_deployed" => Ok(PluginHealthStatus::NotDeployed),
        "pending" => Ok(PluginHealthStatus::Pending),
        "running" => Ok(PluginHealthStatus::Running),
        "dead" => Ok(PluginHealthStatus::Dead),
        value => Err(serde::de::Error::unknown_variant(
            value,
            PLUGIN_HEALTH_EXPECTED,
        )),
    }
}
