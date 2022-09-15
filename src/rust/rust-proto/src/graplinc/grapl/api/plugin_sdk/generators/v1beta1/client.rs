use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};
use generator_service_client::GeneratorServiceClient as GeneratorServiceClientProto;

pub use crate::protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1::generator_service_client;
use crate::{
    client_factory::services::GeneratorClientConfig,
    client_macros::RpcConfig,
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::plugin_sdk::generators::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1 as proto,
    protocol::{
        endpoint::Endpoint,
        error::GrpcClientError,
        service_client::{
            ConfigConnectable,
            ConnectError,
            Connectable,
        },
    },
};

#[derive(Clone)]
pub struct GeneratorServiceClient {
    executor: Executor,
    proto_client: GeneratorServiceClientProto<tonic::transport::Channel>,
}
#[async_trait::async_trait]
impl Connectable for GeneratorServiceClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_sdk.generators.v1beta1.GeneratorService";

    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            GeneratorServiceClientProto<tonic::transport::Channel>,
            endpoint,
        );
        Ok(GeneratorServiceClient {
            executor,
            proto_client,
        })
    }
}

impl ConfigConnectable for GeneratorServiceClient {
    type Config = GeneratorClientConfig;
}

impl GeneratorServiceClient {
    #[tracing::instrument(skip(self, request), err)]
    pub async fn run_generator(
        &mut self,
        request: native::RunGeneratorRequest,
    ) -> Result<native::RunGeneratorResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            run_generator,
            proto::RunGeneratorRequest,
            native::RunGeneratorResponse,
            RpcConfig::default(),
        )
    }
}
