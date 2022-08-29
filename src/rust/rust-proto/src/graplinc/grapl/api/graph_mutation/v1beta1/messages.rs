use crate::{
    graplinc::grapl::{
        api::graph::v1beta1::NodeProperty,
        common::v1beta1::types::{
            EdgeName,
            NodeType,
            PropertyName,
            Uid,
        },
    },
    protobufs::graplinc::grapl::api::graph_mutation::v1beta1::{
        CreateEdgeRequest as CreateEdgeRequestProto,
        CreateEdgeResponse as CreateEdgeResponseProto,
        CreateNodeRequest as CreateNodeRequestProto,
        CreateNodeResponse as CreateNodeResponseProto,
        MutationRedundancy as MutationRedundancyProto,
        SetNodePropertyRequest as SetNodePropertyRequestProto,
        SetNodePropertyResponse as SetNodePropertyResponseProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MutationRedundancy {
    True,
    False,
    Maybe,
}

impl TryFrom<MutationRedundancyProto> for MutationRedundancy {
    type Error = SerDeError;
    fn try_from(v: MutationRedundancyProto) -> Result<MutationRedundancy, Self::Error> {
        match v {
            // It should always be specified, but if it isn't we can still safely fall
            // back to "Maybe"
            MutationRedundancyProto::Unspecified => Ok(MutationRedundancy::Maybe),
            MutationRedundancyProto::True => Ok(MutationRedundancy::True),
            MutationRedundancyProto::False => Ok(MutationRedundancy::False),
            MutationRedundancyProto::Maybe => Ok(MutationRedundancy::Maybe),
        }
    }
}

impl From<MutationRedundancy> for MutationRedundancyProto {
    fn from(v: MutationRedundancy) -> MutationRedundancyProto {
        match v {
            MutationRedundancy::True => MutationRedundancyProto::True,
            MutationRedundancy::False => MutationRedundancyProto::False,
            MutationRedundancy::Maybe => MutationRedundancyProto::Maybe,
        }
    }
}

impl type_url::TypeUrl for MutationRedundancy {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.MutationRedundancy";
}

#[derive(Debug, Clone, PartialEq)]
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
        let tenant_id = proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?
            .into();
        let uid = proto
            .uid
            .ok_or(SerDeError::MissingField("uid"))?
            .try_into()?;
        let property_name = proto
            .property_name
            .ok_or(SerDeError::MissingField("property_name"))?
            .try_into()?;
        let property = proto
            .property
            .ok_or(SerDeError::MissingField("property"))?
            .try_into()?;
        let node_type = proto
            .node_type
            .ok_or(SerDeError::MissingField("node_type"))?
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

#[derive(Debug, Clone, PartialEq)]
pub struct SetNodePropertyResponse {
    pub mutation_redundancy: MutationRedundancy,
}

impl TryFrom<SetNodePropertyResponseProto> for SetNodePropertyResponse {
    type Error = SerDeError;
    fn try_from(proto: SetNodePropertyResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            mutation_redundancy: proto.mutation_redundancy().try_into()?,
        })
    }
}

impl From<SetNodePropertyResponse> for SetNodePropertyResponseProto {
    fn from(value: SetNodePropertyResponse) -> Self {
        let mutation_redundancy: MutationRedundancyProto = value.mutation_redundancy.into();
        Self {
            mutation_redundancy: mutation_redundancy as i32,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateEdgeRequest {
    pub edge_name: EdgeName,
    pub tenant_id: uuid::Uuid,
    pub from_uid: Uid,
    pub to_uid: Uid,
    pub source_node_type: NodeType,
}

impl TryFrom<CreateEdgeRequestProto> for CreateEdgeRequest {
    type Error = SerDeError;
    fn try_from(proto: CreateEdgeRequestProto) -> Result<Self, Self::Error> {
        let edge_name = proto
            .edge_name
            .ok_or(SerDeError::MissingField("edge_name"))?
            .try_into()?;
        let tenant_id = proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?
            .into();
        let from_uid = proto
            .from_uid
            .ok_or(SerDeError::MissingField("from_uid"))?
            .try_into()?;
        let to_uid = proto
            .to_uid
            .ok_or(SerDeError::MissingField("to_uid"))?
            .try_into()?;

        let source_node_type = proto
            .source_node_type
            .ok_or(SerDeError::MissingField("source_node_type"))?
            .try_into()?;

        Ok(Self {
            edge_name,
            tenant_id,
            from_uid,
            to_uid,
            source_node_type,
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
            source_node_type: Some(value.source_node_type.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateEdgeResponse {
    pub mutation_redundancy: MutationRedundancy,
}

impl TryFrom<CreateEdgeResponseProto> for CreateEdgeResponse {
    type Error = SerDeError;
    fn try_from(proto: CreateEdgeResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            mutation_redundancy: proto.mutation_redundancy().try_into()?,
        })
    }
}

impl From<CreateEdgeResponse> for CreateEdgeResponseProto {
    fn from(value: CreateEdgeResponse) -> Self {
        let mutation_redundancy: MutationRedundancyProto = value.mutation_redundancy.into();
        Self {
            mutation_redundancy: mutation_redundancy as i32,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNodeRequest {
    pub tenant_id: uuid::Uuid,
    pub node_type: NodeType,
}

impl TryFrom<CreateNodeRequestProto> for CreateNodeRequest {
    type Error = SerDeError;
    fn try_from(proto: CreateNodeRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?
            .into();

        let node_type = proto
            .node_type
            .ok_or(SerDeError::MissingField("node_type"))?
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

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNodeResponse {
    pub uid: Uid,
}

impl TryFrom<CreateNodeResponseProto> for CreateNodeResponse {
    type Error = SerDeError;
    fn try_from(proto: CreateNodeResponseProto) -> Result<Self, Self::Error> {
        let uid = proto
            .uid
            .ok_or(SerDeError::MissingField("uid"))?
            .try_into()?;
        Ok(Self { uid })
    }
}

impl From<CreateNodeResponse> for CreateNodeResponseProto {
    fn from(value: CreateNodeResponse) -> Self {
        Self {
            uid: Some(value.uid.into()),
        }
    }
}

impl serde_impl::ProtobufSerializable for SetNodePropertyRequest {
    type ProtobufMessage = SetNodePropertyRequestProto;
}

impl type_url::TypeUrl for SetNodePropertyRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.SetNodePropertyRequest";
}

impl serde_impl::ProtobufSerializable for SetNodePropertyResponse {
    type ProtobufMessage = SetNodePropertyResponseProto;
}

impl type_url::TypeUrl for SetNodePropertyResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.SetNodePropertyResponse";
}

impl serde_impl::ProtobufSerializable for CreateEdgeRequest {
    type ProtobufMessage = CreateEdgeRequestProto;
}

impl type_url::TypeUrl for CreateEdgeRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.CreateEdgeRequest";
}

impl serde_impl::ProtobufSerializable for CreateEdgeResponse {
    type ProtobufMessage = CreateEdgeResponseProto;
}

impl type_url::TypeUrl for CreateEdgeResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.CreateEdgeResponse";
}

impl serde_impl::ProtobufSerializable for CreateNodeRequest {
    type ProtobufMessage = CreateNodeRequestProto;
}

impl type_url::TypeUrl for CreateNodeRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.CreateNodeRequest";
}

impl serde_impl::ProtobufSerializable for CreateNodeResponse {
    type ProtobufMessage = CreateNodeResponseProto;
}

impl type_url::TypeUrl for CreateNodeResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph_mutation.v1beta1.CreateNodeResponse\
    ";
}
