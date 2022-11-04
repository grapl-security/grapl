use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginHealthStatus;

#[derive(Debug)]
pub struct CheckHealthResponse(pub Vec<CheckHealthResponseElem>);

impl From<CheckHealthResponse> for PluginHealthStatus {
    fn from(resp: CheckHealthResponse) -> Self {
        if resp.0.is_empty() {
            // No such service found - it may still be booting up.
            // There may be a more accurate status here like "NotFoundYet"
            // but Pending seems okay for now.
            return PluginHealthStatus::Pending;
        }

        if resp.0.iter().any(|elem| elem.status == "critical") {
            // perhaps a more accurate status would be "RunningUnhealthy".
            return PluginHealthStatus::Dead;
        }

        if resp.0.iter().all(|elem| elem.status == "passing") {
            return PluginHealthStatus::Running;
        }

        unreachable!("I don't think there's any case not covered by the above! {resp:?}")
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct CheckHealthResponseElem {
    #[serde(alias = "Status")]
    pub status: String,
    #[serde(alias = "ServiceName")]
    pub service_name: String,
}
