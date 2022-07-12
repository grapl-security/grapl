use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginHealthStatus;

use crate::{
    db::client::PluginRegistryDbClient,
    nomad::client::NomadClient,
};

pub fn get_plugin_health(
    client: &NomadClient,
    db_client: &PluginRegistryDbClient,
    plugin_id: uuid::Uuid,
) -> PluginHealthStatus {
    todo!()
}
