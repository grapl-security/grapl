use bytes::Bytes;

use crate::{
    graplinc::grapl::common::v1beta1::types::{
        EdgeName,
        NodeType,
    },
    protobufs::graplinc::grapl::api::schema_manager::v1beta1::{
        DeployModelRequest as DeployModelRequestProto,
        DeployModelResponse as DeployModelResponseProto,
        EdgeCardinality as EdgeCardinalityProto,
        GetEdgeSchemaRequest as GetEdgeSchemaRequestProto,
        GetEdgeSchemaResponse as GetEdgeSchemaResponseProto,
        SchemaType as SchemaTypeProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaType {
    GraphqlV0,
}

impl TryFrom<SchemaTypeProto> for SchemaType {
    type Error = SerDeError;

    fn try_from(response_proto: SchemaTypeProto) -> Result<Self, Self::Error> {
        match response_proto {
            SchemaTypeProto::GraphqlV0 => Ok(SchemaType::GraphqlV0),
            SchemaTypeProto::Unspecified => Err(SerDeError::UnknownVariant("SchemaType")),
        }
    }
}

impl From<SchemaType> for SchemaTypeProto {
    fn from(response: SchemaType) -> Self {
        match response {
            SchemaType::GraphqlV0 => SchemaTypeProto::GraphqlV0,
        }
    }
}

impl type_url::TypeUrl for SchemaType {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.schema_manager.v1beta1.SchemaType";
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeployModelRequest {
    pub tenant_id: uuid::Uuid,
    pub schema: Bytes,
    pub schema_type: SchemaType,
    pub schema_version: u32,
}

impl TryFrom<DeployModelRequestProto> for DeployModelRequest {
    type Error = SerDeError;

    fn try_from(response_proto: DeployModelRequestProto) -> Result<Self, Self::Error> {
        let schema_type = response_proto.schema_type().try_into()?;
        let schema = response_proto.schema;
        if schema.is_empty() {
            return Err(SerDeError::InvalidField {
                field_name: "schema",
                assertion: "must not be empty".to_owned(),
            });
        }

        let tenant_id = response_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("DeployModelRequest.tenant_id"))?
            .into();

        Ok(DeployModelRequest {
            tenant_id,
            schema,
            schema_type,
            schema_version: response_proto.schema_version,
        })
    }
}

impl From<DeployModelRequest> for DeployModelRequestProto {
    fn from(response: DeployModelRequest) -> Self {
        DeployModelRequestProto {
            tenant_id: Some(response.tenant_id.into()),
            schema_type: response.schema_type as i32,
            schema: response.schema,
            schema_version: response.schema_version,
        }
    }
}

impl type_url::TypeUrl for DeployModelRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.schema_manager.v1beta1.DeployModelRequest";
}

impl serde_impl::ProtobufSerializable for DeployModelRequest {
    type ProtobufMessage = DeployModelRequestProto;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeployModelResponse {}

impl TryFrom<DeployModelResponseProto> for DeployModelResponse {
    type Error = SerDeError;

    fn try_from(response_proto: DeployModelResponseProto) -> Result<Self, Self::Error> {
        let DeployModelResponseProto {} = response_proto;
        Ok(DeployModelResponse {})
    }
}

impl From<DeployModelResponse> for DeployModelResponseProto {
    fn from(response: DeployModelResponse) -> Self {
        let DeployModelResponse {} = response;
        DeployModelResponseProto {}
    }
}

impl type_url::TypeUrl for DeployModelResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.schema_manager.v1beta1.DeployModelResponse";
}

impl serde_impl::ProtobufSerializable for DeployModelResponse {
    type ProtobufMessage = DeployModelResponseProto;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetEdgeSchemaRequest {
    pub tenant_id: uuid::Uuid,
    pub node_type: NodeType,
    pub edge_name: EdgeName,
}

impl TryFrom<GetEdgeSchemaRequestProto> for GetEdgeSchemaRequest {
    type Error = SerDeError;

    fn try_from(response_proto: GetEdgeSchemaRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = response_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("GetEdgeSchemaRequest.tenant_id"))?
            .into();

        let node_type = response_proto
            .node_type
            .ok_or(SerDeError::MissingField("GetEdgeSchemaRequest.node_type"))?
            .try_into()?;

        let edge_name = response_proto
            .edge_name
            .ok_or(SerDeError::MissingField("GetEdgeSchemaRequest.edge_name"))?
            .try_into()?;

        Ok(GetEdgeSchemaRequest {
            tenant_id,
            node_type,
            edge_name,
        })
    }
}

impl From<GetEdgeSchemaRequest> for GetEdgeSchemaRequestProto {
    fn from(response: GetEdgeSchemaRequest) -> Self {
        GetEdgeSchemaRequestProto {
            tenant_id: Some(response.tenant_id.into()),
            node_type: Some(response.node_type.into()),
            edge_name: Some(response.edge_name.into()),
        }
    }
}

impl type_url::TypeUrl for GetEdgeSchemaRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.schema_manager.v1beta1.GetEdgeSchemaRequest";
}

impl serde_impl::ProtobufSerializable for GetEdgeSchemaRequest {
    type ProtobufMessage = GetEdgeSchemaRequestProto;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetEdgeSchemaResponse {
    pub reverse_edge_name: EdgeName,
    pub cardinality: EdgeCardinality,
    pub reverse_cardinality: EdgeCardinality,
}

impl TryFrom<GetEdgeSchemaResponseProto> for GetEdgeSchemaResponse {
    type Error = SerDeError;

    fn try_from(response_proto: GetEdgeSchemaResponseProto) -> Result<Self, Self::Error> {
        let cardinality = response_proto.cardinality().try_into()?;
        let reverse_cardinality = response_proto.reverse_cardinality().try_into()?;

        let reverse_edge_name = response_proto
            .reverse_edge_name
            .ok_or(SerDeError::MissingField(
                "GetEdgeSchemaResponse.reverse_edge_name",
            ))?
            .try_into()?;

        Ok(GetEdgeSchemaResponse {
            reverse_edge_name,
            cardinality,
            reverse_cardinality,
        })
    }
}

impl From<GetEdgeSchemaResponse> for GetEdgeSchemaResponseProto {
    fn from(response: GetEdgeSchemaResponse) -> Self {
        GetEdgeSchemaResponseProto {
            reverse_edge_name: Some(response.reverse_edge_name.into()),
            cardinality: response.cardinality as i32,
            reverse_cardinality: response.reverse_cardinality as i32,
        }
    }
}

impl type_url::TypeUrl for GetEdgeSchemaResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.schema_manager.v1beta1.GetEdgeSchemaResponse";
}

impl serde_impl::ProtobufSerializable for GetEdgeSchemaResponse {
    type ProtobufMessage = GetEdgeSchemaResponseProto;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EdgeCardinality {
    ToOne,
    ToMany,
}

impl TryFrom<EdgeCardinalityProto> for EdgeCardinality {
    type Error = SerDeError;

    fn try_from(response_proto: EdgeCardinalityProto) -> Result<Self, Self::Error> {
        match response_proto {
            EdgeCardinalityProto::ToOne => Ok(EdgeCardinality::ToOne),
            EdgeCardinalityProto::ToMany => Ok(EdgeCardinality::ToMany),
            EdgeCardinalityProto::Unspecified => Err(SerDeError::UnknownVariant("EdgeCardinality")),
        }
    }
}

impl From<EdgeCardinality> for EdgeCardinalityProto {
    fn from(response: EdgeCardinality) -> Self {
        match response {
            EdgeCardinality::ToOne => EdgeCardinalityProto::ToOne,
            EdgeCardinality::ToMany => EdgeCardinalityProto::ToMany,
        }
    }
}

impl type_url::TypeUrl for EdgeCardinality {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.schema_manager.v1beta1.EdgeCardinality";
}
