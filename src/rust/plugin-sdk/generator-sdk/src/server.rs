use rust_proto::plugin_sdk::generators::{generator_service_server::{
    GeneratorService,
    GeneratorServiceServer,
}, GeneratedGraphProto, RunGeneratorRequestProto, RunGeneratorResponseProto, GeneratorsDeserializationError};
pub use rust_proto::{
    graph_descriptions::GraphDescription,
    plugin_sdk::generators::RunGeneratorRequest,
};
use tonic::transport::{
    Identity,
    ServerTlsConfig,
};

/// This trait is the main interface for Grapl Generator Plugins.
/// Generators should be very straightforward - essentially they should just be parsers.
/// Implementations of this trait should be passed into `exec_service`, which will
/// serve the generator via gRPC.
pub trait GraphGenerator {
    type Error: std::error::Error;

    fn run_generator(&self, data: RunGeneratorRequest) -> Result<GraphDescription, Self::Error>;
}

// This is only public so that it can show up in an `impl Into<GraphGeneratorImpl>` used in
// a public interface
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
        let request: RunGeneratorRequest = request.try_into()
            .map_err(|e: GeneratorsDeserializationError| {
                tracing::error!(message="Invalid RunGeneratorRequest", e=?e);
                tonic::Status::invalid_argument(e.to_string())
            })?;

        let graph_description = self.0.run_generator(request)
            // In the future we can ask implementations to give more information about their error
            .map_err(|e| {
                tracing::error!(message="Generator failed", e=?e);
                tonic::Status::unknown(e.to_string())
            })?;

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
    //       the right way to get these configuration values passed around
    let cert = tokio::fs::read("/etc/ssl/private/plugin-client-cert.pem").await?;
    let key = tokio::fs::read("/etc/ssl/private/plugin-client-cert.key").await?;

    let identity = Identity::from_pem(cert, key);

    tonic::transport::Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .trace_fn(|request| {
            tracing::info_span!(
                "exec_service",
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
