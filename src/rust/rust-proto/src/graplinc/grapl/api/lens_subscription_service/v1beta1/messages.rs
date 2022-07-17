#![allow(warnings)]

use uuid::Uuid;
use crate::graplinc::grapl::common::v1beta1::types::{NodeType, Uid};
use crate::protobufs::graplinc::grapl::api::lens_subscription_service::v1beta1::{lens_update, LensUpdate as LensUpdateProto, NodeAddedToLensScope as NodeAddedToLensScopeProto, NodeRemovedFromLensScope as NodeRemovedFromLensScopeProto,
                                                                                 SubscribeToLensRequest as SubscribeToLensRequestProto,
                                                                                 SubscribeToLensResponse as SubscribeToLensResponseProto,
};

use crate::{serde_impl, SerDeError};
use crate::type_url::TypeUrl;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeAddedToLensScope {
    /// The tenant that the operation pertains to
    pub tenant_id: Uuid,
    /// The uid of the lens that had its scope updated
    pub lens_uid: Uid,
    /// The uid of the node added to the lens's scope
    pub node_uid: Uid,
    /// The type of the node added to the lens's scope
    pub node_type: NodeType,
}

impl TryFrom<NodeAddedToLensScopeProto> for NodeAddedToLensScope {
    type Error = SerDeError;
    fn try_from(value: NodeAddedToLensScopeProto) -> Result<Self, Self::Error> {
        Ok(Self {
            tenant_id: value.tenant_id.ok_or(SerDeError::MissingField("tenant_id"))?
                .into(),
            lens_uid: value.lens_uid.ok_or(SerDeError::MissingField("lens_uid"))?
                .try_into()?,
            node_uid: value.node_uid.ok_or(SerDeError::MissingField("node_uid"))?
                .try_into()?,
            node_type: value.node_type.ok_or(SerDeError::MissingField("node_type"))?
                .try_into()?,
        })
    }
}

impl From<NodeAddedToLensScope> for NodeAddedToLensScopeProto {
    fn from(value: NodeAddedToLensScope) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            lens_uid: Some(value.lens_uid.into()),
            node_uid: Some(value.node_uid.into()),
            node_type: Some(value.node_type.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeRemovedFromLensScope {
    /// The tenant that the operation pertains to
    pub tenant_id: Uuid,
    /// The uid of the lens that had its scope updated
    pub lens_uid: Uid,
    /// The uid of the node removed from the lens's scope
    pub node_uid: Uid,
    /// The type of the node removed from the lens's scope
    pub node_type: NodeType,
}

impl TryFrom<NodeRemovedFromLensScopeProto> for NodeRemovedFromLensScope {
    type Error = SerDeError;
    fn try_from(value: NodeRemovedFromLensScopeProto) -> Result<Self, Self::Error> {
        Ok(Self {
            tenant_id: value.tenant_id.ok_or(SerDeError::MissingField("tenant_id"))?
                .into(),
            lens_uid: value.lens_uid.ok_or(SerDeError::MissingField("lens_uid"))?
                .try_into()?,
            node_uid: value.node_uid.ok_or(SerDeError::MissingField("node_uid"))?
                .try_into()?,
            node_type: value.node_type.ok_or(SerDeError::MissingField("node_type"))?
                .try_into()?,
        })
    }
}

impl From<NodeRemovedFromLensScope> for NodeRemovedFromLensScopeProto {
    fn from(value: NodeRemovedFromLensScope) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            lens_uid: Some(value.lens_uid.into()),
            node_uid: Some(value.node_uid.into()),
            node_type: Some(value.node_type.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LensUpdate {
    NodeAddedToLensScope(NodeAddedToLensScope),
    NodeRemovedFromLensScope(NodeRemovedFromLensScope),
}

impl TryFrom<LensUpdateProto> for LensUpdate{
    type Error = SerDeError;

    fn try_from(value: LensUpdateProto) -> Result<Self, Self::Error> {
        match value.inner {
            Some(lens_update::Inner::NodeAddedToLensScope(inner)) =>
                Ok(Self::NodeAddedToLensScope(inner.try_into()?)),
            Some(lens_update::Inner::NodeRemovedFromLensScope(inner)) =>
                Ok(Self::NodeRemovedFromLensScope(inner.try_into()?)),
            None => Err(SerDeError::MissingField("inner")),
        }
    }
}

impl From<LensUpdate> for LensUpdateProto {
    fn from(value: LensUpdate) -> Self {
        match value {
            LensUpdate::NodeAddedToLensScope(inner) => {
                Self {
                    inner: Some(
                        lens_update::Inner::NodeAddedToLensScope(inner.into())
                    )
                }
            }
            LensUpdate::NodeRemovedFromLensScope(inner) => {
                Self {
                    inner: Some(
                        lens_update::Inner::NodeRemovedFromLensScope(inner.into())
                    )
                }
            }
        }
    }
}

impl serde_impl::ProtobufSerializable for LensUpdate {
    type ProtobufMessage = LensUpdateProto;
}

impl TypeUrl for LensUpdate {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.lens_subscription_service.v1beta1.LensUpdate";
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscribeToLensRequest {
    pub tenant_id: Uuid,
    pub lens_uid: Uid,
}

impl TryFrom<SubscribeToLensRequestProto> for SubscribeToLensRequest {
    type Error = SerDeError;
    fn try_from(value: SubscribeToLensRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            tenant_id: value.tenant_id.ok_or(SerDeError::MissingField("tenant_id"))?
                .into(),
            lens_uid: value.lens_uid.ok_or(SerDeError::MissingField("lens_uid"))?
                .try_into()?,
        })
    }
}

impl From<SubscribeToLensRequest> for SubscribeToLensRequestProto {
    fn from(value: SubscribeToLensRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            lens_uid: Some(value.lens_uid.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscribeToLensResponse {
    pub lens_update: LensUpdate,
    pub update_offset: u64,
}


impl TryFrom<SubscribeToLensResponseProto> for SubscribeToLensResponse {
    type Error = SerDeError;
    fn try_from(value: SubscribeToLensResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            lens_update: value.lens_update.ok_or(SerDeError::MissingField("lens_update"))?
                .try_into()?,
            update_offset: value.update_offset,
        })
    }
}

impl From<SubscribeToLensResponse> for SubscribeToLensResponseProto {
    fn from(value: SubscribeToLensResponse) -> Self {
        Self {
            lens_update: Some(value.lens_update.into()),
            update_offset: value.update_offset,
        }
    }
}
