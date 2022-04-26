#![allow(warnings)]

use crate::SerDeError;

use crate::graplinc::grapl::api::graph::v1beta1::NodeProperty;
use crate::protobufs::graplinc::grapl::api::graph_mutation::v1beta1::{
    PropertyName as PropertyNameProto,
    EdgeName as EdgeNameProto,
    NodeType as NodeTypeProto,
    Uid as UidProto,
    SetNodePropertyRequest as SetNodePropertyRequestProto,
    SetNodePropertyResponse as SetNodePropertyResponseProto,
    CreateEdgeRequest as CreateEdgeRequestProto,
    CreateEdgeResponse as CreateEdgeResponseProto,
    CreateNodeRequest as CreateNodeRequestProto,
    CreateNodeResponse as CreateNodeResponseProto,
};
use crate::type_url;

pub struct PropertyName {
    pub value: String,
}

impl TryFrom<PropertyNameProto> for PropertyName {
    type Error = SerDeError;
    fn try_from(proto: PropertyNameProto) -> Result<Self, Self::Error> {
        Ok(Self {
            value: proto.value,
        })
    }
}

impl From<PropertyName> for PropertyNameProto {
    fn from(value: PropertyName) -> Self {
        Self {
            value: value.value,
        }
    }
}

pub struct EdgeName {
    pub value: String,
}

impl TryFrom<EdgeNameProto> for EdgeName {
    type Error = SerDeError;
    fn try_from(proto: EdgeNameProto) -> Result<Self, Self::Error> {
        Ok(Self {
            value: proto.value,
        })
    }
}

impl From<EdgeName> for EdgeNameProto {
    fn from(value: EdgeName) -> Self {
        Self {
            value: value.value,
        }
    }
}

pub struct NodeType {
    pub value: String,
}

impl TryFrom<NodeTypeProto> for NodeType {
    type Error = SerDeError;
    fn try_from(proto: NodeTypeProto) -> Result<Self, Self::Error> {
        Ok(Self {
            value: proto.value,
        })
    }
}

impl From<NodeType> for NodeTypeProto {
    fn from(value: NodeType) -> Self {
        Self {
            value: value.value,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Uid {
    pub value: u64,
}

impl Uid {
    pub fn as_i64(&self) -> i64 {
        self.value as i64
    }
}

impl TryFrom<UidProto> for Uid {
    type Error = SerDeError;
    fn try_from(proto: UidProto) -> Result<Self, Self::Error> {
        Ok(Self {
            value: proto.value,
        })
    }
}

impl From<Uid> for UidProto {
    fn from(value: Uid) -> Self {
        Self {
            value: value.value,
        }
    }
}

pub struct SetNodePropertyRequest {
    pub tenant_id: uuid::Uuid,
    pub uid: Uid,
    pub node_type: NodeType,
    pub property_name: PropertyName,
    pub property: NodeProperty,
}

impl TryFrom<SetNodePropertyRequestProto> for SetNodePropertyRequest {
    type Error = SerDeError;
    fn try_from(proto: SetNodePropertyRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = proto.tenant_id.ok_or(SerDeError::MissingField("tenant_id"))?
            .into();
        let uid = proto.uid.ok_or(SerDeError::MissingField("uid"))?
            .try_into()?;
        let property_name = proto.property_name.ok_or(SerDeError::MissingField("property_name"))?
            .try_into()?;
        let property = proto.property.ok_or(SerDeError::MissingField("property"))?
            .try_into()?;
        let node_type = proto.node_type.ok_or(SerDeError::MissingField("node_type"))?
            .try_into()?;
        Ok(Self {
            tenant_id,
            uid,
            property_name,
            node_type,
            property,
        })
    }
}

impl From<SetNodePropertyRequest> for SetNodePropertyRequestProto {
    fn from(value: SetNodePropertyRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            uid: Some(value.uid.into()),
            node_type: Some(value.node_type.into()),
            property_name: Some(value.property_name.into()),
            property: Some(value.property.into()),
        }
    }
}

pub struct SetNodePropertyResponse {
    pub was_redundant: bool,
}

impl TryFrom<SetNodePropertyResponseProto> for SetNodePropertyResponse {
    type Error = SerDeError;
    fn try_from(proto: SetNodePropertyResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            was_redundant: proto.was_redundant,
        })
    }
}

impl From<SetNodePropertyResponse> for SetNodePropertyResponseProto {
    fn from(value: SetNodePropertyResponse) -> Self {
        Self {
            was_redundant: value.was_redundant,
        }
    }
}

pub struct CreateEdgeRequest {
    pub edge_name: EdgeName,
    pub tenant_id: uuid::Uuid,
    pub from_uid: Uid,
    pub to_uid: Uid,
}

impl TryFrom<CreateEdgeRequestProto> for CreateEdgeRequest {
    type Error = SerDeError;
    fn try_from(proto: CreateEdgeRequestProto) -> Result<Self, Self::Error> {
        let edge_name = proto.edge_name.ok_or(SerDeError::MissingField("edge_name"))?
            .try_into()?;
        let tenant_id = proto.tenant_id.ok_or(SerDeError::MissingField("tenant_id"))?
            .into();
        let from_uid = proto.from_uid.ok_or(SerDeError::MissingField("from_uid"))?
            .try_into()?;
        let to_uid = proto.to_uid.ok_or(SerDeError::MissingField("to_uid"))?
            .try_into()?;
        Ok(Self {
            edge_name,
            tenant_id,
            from_uid,
            to_uid,
        })
    }
}

impl From<CreateEdgeRequest> for CreateEdgeRequestProto {
    fn from(value: CreateEdgeRequest) -> Self {
        Self {
            edge_name: Some(value.edge_name.into()),
            tenant_id: Some(value.tenant_id.into()),
            from_uid: Some(value.from_uid.into()),
            to_uid: Some(value.to_uid.into()),
        }
    }
}

pub struct CreateEdgeResponse {
    pub was_redundant: bool,
}

impl TryFrom<CreateEdgeResponseProto> for CreateEdgeResponse {
    type Error = SerDeError;
    fn try_from(proto: CreateEdgeResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            was_redundant: proto.was_redundant,
        })
    }
}

impl From<CreateEdgeResponse> for CreateEdgeResponseProto {
    fn from(value: CreateEdgeResponse) -> Self {
        Self {
            was_redundant: value.was_redundant,
        }
    }
}

pub struct CreateNodeRequest {
    pub tenant_id: uuid::Uuid,
    pub node_type: NodeType,
}

impl TryFrom<CreateNodeRequestProto> for CreateNodeRequest {
    type Error = SerDeError;
    fn try_from(proto: CreateNodeRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = proto.tenant_id.ok_or(SerDeError::MissingField("tenant_id"))?
            .into();
        let node_type = proto.node_type.ok_or(SerDeError::MissingField("node_type"))?
            .try_into()?;
        Ok(Self {
            tenant_id,
            node_type,
        })
    }
}

impl From<CreateNodeRequest> for CreateNodeRequestProto {
    fn from(value: CreateNodeRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            node_type: Some(value.node_type.into()),
        }
    }
}

pub struct CreateNodeResponse {
    pub uid: Uid,
}

impl TryFrom<CreateNodeResponseProto> for CreateNodeResponse {
    type Error = SerDeError;
    fn try_from(proto: CreateNodeResponseProto) -> Result<Self, Self::Error> {
        let uid = proto.uid.ok_or(SerDeError::MissingField("uid"))?
            .try_into()?;
        Ok(Self {
            uid,
        })
    }
}

impl From<CreateNodeResponse> for CreateNodeResponseProto {
    fn from(value: CreateNodeResponse) -> Self {
        Self {
            uid: Some(value.uid.into()),
        }
    }
}

impl type_url::TypeUrl for PropertyName {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.PropertyName";
}

impl type_url::TypeUrl for EdgeName {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.EdgeName";
}

impl type_url::TypeUrl for NodeType {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.NodeType";
}

impl type_url::TypeUrl for Uid {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.Uid";
}

impl type_url::TypeUrl for SetNodePropertyRequest {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.SetNodePropertyRequest";
}

impl type_url::TypeUrl for SetNodePropertyResponse {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.SetNodePropertyResponse";
}

impl type_url::TypeUrl for CreateEdgeRequest {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.CreateEdgeRequest";
}

impl type_url::TypeUrl for CreateEdgeResponse {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.CreateEdgeResponse";
}

impl type_url::TypeUrl for CreateNodeRequest {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.CreateNodeRequest";
}

impl type_url::TypeUrl for CreateNodeResponse {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.CreateNodeResponse\
    ";
}