use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};

use crate::{
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::{
        client_factory::services::PipelineIngressClientConfig,
        client_macros::RpcConfig,
        pipeline_ingress::v1beta1 as native,
        protocol::{
            endpoint::Endpoint,
            error::GrpcClientError,
            service_client::{
                ConnectError,
                Connectable,
            },
        },
    },
    protobufs::graplinc::grapl::api::pipeline_ingress::{
        v1beta1 as proto,
        v1beta1::pipeline_ingress_service_client::PipelineIngressServiceClient as PipelineIngressServiceClientProto,
    },
};

pub type PipelineIngressClientError = GrpcClientError;

pub struct PipelineIngressClient {
    executor: Executor,
    proto_client: PipelineIngressServiceClientProto<tonic::transport::Channel>,
}

#[async_trait::async_trait]
impl Connectable for PipelineIngressClient {
    type Config = PipelineIngressClientConfig;
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.pipeline_ingress.v1beta1.PipelineIngressService";

    #[tracing::instrument(err)]
    async fn connect_with_endpoint(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            PipelineIngressServiceClientProto<tonic::transport::Channel>,
            endpoint,
        );

        Ok(Self {
            executor,
            proto_client,
        })
    }
}

impl PipelineIngressClient {
    pub async fn publish_raw_log(
        &mut self,
        request: native::PublishRawLogRequest,
    ) -> Result<native::PublishRawLogResponse, PipelineIngressClientError> {
        execute_client_rpc!(
            self,
            request,
            publish_raw_log,
            proto::PublishRawLogRequest,
            native::PublishRawLogResponse,
            RpcConfig::default(),
        )
    }
}
