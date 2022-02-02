use crate::graph_descriptions::GraphDescription;
pub use crate::graplinc::grapl::api::plugin_sdk::generators::v1beta1::{
    generator_service_client,
    generator_service_server,
    GeneratedGraph as GeneratedGraphProto,
    RunGeneratorRequest as RunGeneratorRequestProto,
    RunGeneratorResponse as RunGeneratorResponseProto,
};

#[derive(Debug, thiserror::Error)]
pub enum GeneratorsDeserializationError {
    #[error("Missing a required field {0}")]
    MissingRequiredField(&'static str),
    #[error("Empty field {0}")]
    EmptyField(&'static str),
    #[error("Unknown variant {0}")]
    UnknownVariant(std::borrow::Cow<'static, str>),
}

#[derive(Clone)]
pub struct GeneratedGraph {
    pub graph_description: GraphDescription,
}

impl TryFrom<GeneratedGraphProto> for GeneratedGraph {
    type Error = GeneratorsDeserializationError;

    fn try_from(value: GeneratedGraphProto) -> Result<Self, Self::Error> {
        let graph_description =
            value
                .graph_description
                .ok_or(GeneratorsDeserializationError::MissingRequiredField(
                    "GeneratedGraphProto.graph_description",
                ))?;

        Ok(Self { graph_description })
    }
}

impl From<GeneratedGraph> for GeneratedGraphProto {
    fn from(value: GeneratedGraph) -> Self {
        GeneratedGraphProto {
            graph_description: Some(value.graph_description),
        }
    }
}

#[derive(Clone)]
pub struct RunGeneratorRequest {
    pub data: Vec<u8>,
}

impl TryFrom<RunGeneratorRequestProto> for RunGeneratorRequest {
    type Error = GeneratorsDeserializationError;

    fn try_from(value: RunGeneratorRequestProto) -> Result<Self, Self::Error> {
        if value.data.is_empty() {
            return Err(GeneratorsDeserializationError::EmptyField(
                "RunGeneratorRequest.data",
            ));
        }

        Ok(Self { data: value.data })
    }
}

impl From<RunGeneratorRequest> for RunGeneratorRequestProto {
    fn from(value: RunGeneratorRequest) -> Self {
        RunGeneratorRequestProto { data: value.data }
    }
}

#[derive(Clone)]
pub struct RunGeneratorResponse {
    pub generated_graph: GeneratedGraph,
}

impl TryFrom<RunGeneratorResponseProto> for RunGeneratorResponse {
    type Error = GeneratorsDeserializationError;

    fn try_from(value: RunGeneratorResponseProto) -> Result<Self, Self::Error> {
        let generated_graph = value
            .generated_graph
            .ok_or(GeneratorsDeserializationError::MissingRequiredField(
                "RunGeneratorResponse.graph_description",
            ))?
            .try_into()?;

        Ok(Self { generated_graph })
    }
}

impl From<RunGeneratorResponse> for RunGeneratorResponseProto {
    fn from(value: RunGeneratorResponse) -> Self {
        RunGeneratorResponseProto {
            generated_graph: Some(value.generated_graph.into()),
        }
    }
}
