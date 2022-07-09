pub mod client;
pub mod server;

use bytes::Bytes;

use crate::{
    graplinc::grapl::api::graph::v1beta1::GraphDescription,
    protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1 as proto,
    serde_impl,
    type_url,
    SerDeError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedGraph {
    pub graph_description: GraphDescription,
}

impl TryFrom<proto::GeneratedGraph> for GeneratedGraph {
    type Error = SerDeError;

    fn try_from(value: proto::GeneratedGraph) -> Result<Self, Self::Error> {
        let graph_description: GraphDescription = value
            .graph_description
            .ok_or(SerDeError::MissingField(
                "proto::GeneratedGraph.graph_description",
            ))?
            .try_into()?;

        Ok(Self { graph_description })
    }
}

impl From<GeneratedGraph> for proto::GeneratedGraph {
    fn from(value: GeneratedGraph) -> Self {
        proto::GeneratedGraph {
            graph_description: Some(value.graph_description.into()),
        }
    }
}

impl type_url::TypeUrl for GeneratedGraph {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.generators.v1beta1.GeneratedGraph";
}

impl serde_impl::ProtobufSerializable for GeneratedGraph {
    type ProtobufMessage = proto::GeneratedGraph;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunGeneratorRequest {
    pub data: Bytes,
}

impl TryFrom<proto::RunGeneratorRequest> for RunGeneratorRequest {
    type Error = SerDeError;

    fn try_from(value: proto::RunGeneratorRequest) -> Result<Self, Self::Error> {
        if value.data.is_empty() {
            return Err(SerDeError::MissingField("RunGeneratorRequest.data"));
        }

        Ok(Self { data: value.data })
    }
}

impl From<RunGeneratorRequest> for proto::RunGeneratorRequest {
    fn from(value: RunGeneratorRequest) -> Self {
        proto::RunGeneratorRequest { data: value.data }
    }
}

impl type_url::TypeUrl for RunGeneratorRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.generators.v1beta1.RunGeneratorRequest";
}

impl serde_impl::ProtobufSerializable for RunGeneratorRequest {
    type ProtobufMessage = proto::RunGeneratorRequest;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunGeneratorResponse {
    pub generated_graph: GeneratedGraph,
}

impl TryFrom<proto::RunGeneratorResponse> for RunGeneratorResponse {
    type Error = SerDeError;

    fn try_from(value: proto::RunGeneratorResponse) -> Result<Self, Self::Error> {
        let generated_graph = value
            .generated_graph
            .ok_or(SerDeError::MissingField(
                "RunGeneratorResponse.graph_description",
            ))?
            .try_into()?;

        Ok(Self { generated_graph })
    }
}

impl From<RunGeneratorResponse> for proto::RunGeneratorResponse {
    fn from(value: RunGeneratorResponse) -> Self {
        proto::RunGeneratorResponse {
            generated_graph: Some(value.generated_graph.into()),
        }
    }
}

impl type_url::TypeUrl for RunGeneratorResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.generators.v1beta1.RunGeneratorResponse";
}

impl serde_impl::ProtobufSerializable for RunGeneratorResponse {
    type ProtobufMessage = proto::RunGeneratorResponse;
}
