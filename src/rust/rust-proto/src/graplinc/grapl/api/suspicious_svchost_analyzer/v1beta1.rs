use crate::{
    graplinc::grapl::api::graph::v1beta1::{
        ExecutionHit,
        MergedGraph,
    },
    protobufs::graplinc::grapl::api::{
        graph::v1beta1::{
            ExecutionHit as ExecutionHitProto,
            MergedGraph as MergedGraphProto,
        },
        suspicious_svchost_analyzer::v1beta1::{
            AnalyzeRequest as AnalyzeRequestProto,
            AnalyzeResponse as AnalyzeResponseProto,
        },
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// AnalyzeRequest
//

#[derive(Debug, PartialEq, Clone)]
pub struct AnalyzeRequest {
    pub merged_graph: MergedGraph,
}

impl TryFrom<AnalyzeRequestProto> for AnalyzeRequest {
    type Error = SerDeError;

    fn try_from(analyze_request_proto: AnalyzeRequestProto) -> Result<Self, Self::Error> {
        match analyze_request_proto.merged_graph {
            Some(merged_graph_proto) => Ok(AnalyzeRequest {
                merged_graph: merged_graph_proto.try_into()?,
            }),
            None => Err(SerDeError::MissingField("merged_graph")),
        }
    }
}

impl From<AnalyzeRequest> for AnalyzeRequestProto {
    fn from(analyze_request: AnalyzeRequest) -> Self {
        AnalyzeRequestProto {
            merged_graph: Some(MergedGraphProto::from(analyze_request.merged_graph)),
        }
    }
}

impl type_url::TypeUrl for AnalyzeRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.suspicious_svchost_analyzer.v1beta1.AnalyzeRequest";
}

impl serde_impl::ProtobufSerializable for AnalyzeRequest {
    type ProtobufMessage = AnalyzeRequestProto;
}

//
// AnalyzeResponse
//

#[derive(Debug, PartialEq, Clone)]
pub struct AnalyzeResponse {
    pub execution_hits: Vec<ExecutionHit>,
}

impl TryFrom<AnalyzeResponseProto> for AnalyzeResponse {
    type Error = SerDeError;

    fn try_from(analyze_response_proto: AnalyzeResponseProto) -> Result<Self, Self::Error> {
        let mut execution_hits = Vec::with_capacity(analyze_response_proto.execution_hits.len());
        for execution_hit_proto in analyze_response_proto.execution_hits {
            execution_hits.push(ExecutionHit::try_from(execution_hit_proto)?);
        }

        Ok(AnalyzeResponse { execution_hits })
    }
}

impl From<AnalyzeResponse> for AnalyzeResponseProto {
    fn from(analyze_response: AnalyzeResponse) -> Self {
        let mut execution_hits = Vec::with_capacity(analyze_response.execution_hits.len());
        for execution_hit in analyze_response.execution_hits {
            execution_hits.push(ExecutionHitProto::from(execution_hit));
        }

        AnalyzeResponseProto { execution_hits }
    }
}

impl type_url::TypeUrl for AnalyzeResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.suspicious_svchost_analyzer.v1beta1.AnalyzeResponse";
}

impl serde_impl::ProtobufSerializable for AnalyzeResponse {
    type ProtobufMessage = AnalyzeResponseProto;
}

//
// client
//

pub mod client {
    use futures::FutureExt;
    use thiserror::Error;
    use tonic::Request;
    use crate::{
        graplinc::grapl::api::suspicious_svchost_analyzer::v1beta1::{
            AnalyzeRequest,
            AnalyzeResponse
        },
        protobufs::graplinc::grapl::api::suspicious_svchost_analyzer::v1beta1::suspicious_svchost_analyzer_client::SuspiciousSvchostAnalyzerClient as SuspiciousSvchostAnalyzerClientProto,
        SerDeError,
    };

    #[non_exhaustive]
    #[derive(Debug, Error)]
    pub enum ConfigurationError {
        #[error("failed to connect {0}")]
        ConnectionError(#[from] tonic::transport::Error),
    }

    #[non_exhaustive]
    #[derive(Debug, Error)]
    pub enum SuspiciousSvchostAnalyzerClientError {
        #[error("failed to serialize/deserialize {0}")]
        SerDeError(#[from] SerDeError),

        #[error("received unfavorable gRPC status {0}")]
        GrpcStatus(#[from] tonic::Status),
    }

    pub struct SuspiciousSvchostAnalyzerClient {
        proto_client: SuspiciousSvchostAnalyzerClientProto<tonic::transport::Channel>,
    }

    impl SuspiciousSvchostAnalyzerClient {
        pub async fn connect<T>(endpoint: T) -> Result<Self, ConfigurationError>
        where
            T: std::convert::TryInto<tonic::transport::Endpoint>,
            T::Error: std::error::Error + Send + Sync + 'static,
        {
            Ok(SuspiciousSvchostAnalyzerClient {
                proto_client: SuspiciousSvchostAnalyzerClientProto::connect(endpoint).await?,
            })
        }

        pub async fn analyze(
            &mut self,
            request: AnalyzeRequest,
        ) -> Result<AnalyzeResponse, SuspiciousSvchostAnalyzerClientError> {
            self.proto_client
                .analyze(Request::new(request.into()))
                .map(
                    |response| -> Result<AnalyzeResponse, SuspiciousSvchostAnalyzerClientError> {
                        let inner = response?.into_inner();
                        Ok(inner.try_into()?)
                    },
                )
                .await
        }
    }
}
