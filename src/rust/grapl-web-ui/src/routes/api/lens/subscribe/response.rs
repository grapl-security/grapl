use rust_proto::graplinc::grapl::api::lens_subscription_service::v1beta1::messages::{
    LensUpdate,
    SubscribeToLensResponse,
};

use super::WebResponseError;

#[derive(serde::Serialize, Debug)]
#[serde(tag = "action")]
#[serde(rename_all = "snake_case")]
pub(super) enum WebResponse {
    Added(NodeAddedToLensScope),
    Removed(NodeRemovedFromLensScope),
}

#[derive(serde::Serialize, Debug)]
pub(super) struct NodeAddedToLensScope {
    pub tenant_id: uuid::Uuid,
    pub lens_uid: u64,
    pub node_uid: u64,
    pub node_type: String,
}

#[derive(serde::Serialize, Debug)]
pub(super) struct NodeRemovedFromLensScope {
    pub tenant_id: uuid::Uuid,
    pub lens_uid: u64,
    pub node_uid: u64,
    pub node_type: String,
}

impl From<SubscribeToLensResponse> for WebResponse {
    fn from(proto_response: SubscribeToLensResponse) -> Self {
        match proto_response.lens_update {
            LensUpdate::NodeAddedToLensScope(update) => WebResponse::Added(NodeAddedToLensScope {
                tenant_id: update.tenant_id,
                lens_uid: update.lens_uid.as_u64(),
                node_uid: update.node_uid.as_u64(),
                node_type: update.node_type.value,
            }),
            LensUpdate::NodeRemovedFromLensScope(update) => {
                WebResponse::Removed(NodeRemovedFromLensScope {
                    tenant_id: update.tenant_id,
                    lens_uid: update.lens_uid.as_u64(),
                    node_uid: update.node_uid.as_u64(),
                    node_type: update.node_type.value,
                })
            }
        }
    }
}

impl WebResponse {
    pub(super) fn as_json_bytes(&self) -> Result<actix_web::web::Bytes, WebResponseError> {
        let json_str = serde_json::to_string(&self)?;
        Ok(actix_web::web::Bytes::from(json_str))
    }
}
