use rust_proto::plugin_sdk::generators::{
    generator_service_server::{
        GeneratorService,
        GeneratorServiceServer,
    },
    GeneratedGraphProto,
    RunGeneratorRequestProto,
    RunGeneratorResponseProto,
};
pub use rust_proto::{
    graph_descriptions::GraphDescription,
    plugin_sdk::generators::RunGeneratorRequest,
};
use tonic::transport::{
    Identity,
    ServerTlsConfig,
};

pub trait GraphGenerator {
    type Error: std::error::Error;

    fn run_generator(&self, data: RunGeneratorRequest) -> Result<GraphDescription, Self::Error>;
}

pub struct GraphGeneratorImpl<T>(T)
where
    T: GraphGenerator + Send + Sync + 'static;

impl<T> From<T> for GraphGeneratorImpl<T>
where
    T: GraphGenerator + Send + Sync + 'static,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[tonic::async_trait]
impl<T> GeneratorService for GraphGeneratorImpl<T>
where
    T: GraphGenerator + Send + Sync + 'static,
{
    async fn run_generator(
        &self,
        request: tonic::Request<RunGeneratorRequestProto>,
    ) -> Result<tonic::Response<RunGeneratorResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request: RunGeneratorRequest = request.try_into().expect("todo");
        let graph_description = self.0.run_generator(request).expect("generator failed");

        Ok(tonic::Response::new(RunGeneratorResponseProto {
            generated_graph: Some(GeneratedGraphProto {
                graph_description: Some(graph_description),
            }),
        }))
    }
}

pub async fn exec_service<T>(
    graph_generator: impl Into<GraphGeneratorImpl<T>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: GraphGenerator + Send + Sync + 'static,
{
    let graph_generator = graph_generator.into();
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<GeneratorServiceServer<GraphGeneratorImpl<T>>>()
        .await;

    // todo: When bootstrapping and this service are more mature we should determine
    // the right way to get these configuration values passed around
    let cert = tokio::fs::read("/etc/ssl/private/plugin-client-cert.pem").await?;
    let key = tokio::fs::read("/etc/ssl/private/plugin-client-cert.key").await?;

    let identity = Identity::from_pem(cert, key);

    tonic::transport::Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .trace_fn(|request| {
            tracing::info_span!(
                "PluginWorkQueue",
                headers = ?request.headers(),
                method = ?request.method(),
                uri = %request.uri(),
                extensions = ?request.extensions(),
            )
        })
        .add_service(health_service)
        .add_service(GeneratorServiceServer::new(graph_generator))
        .serve("0.0.0.0:5555".parse()?)
        .await?;

    Ok(())
}
