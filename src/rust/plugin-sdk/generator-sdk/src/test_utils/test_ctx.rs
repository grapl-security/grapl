use std::marker::PhantomData;

use futures::channel::oneshot::Sender;
use rust_proto::{
    graplinc::{
        common::v1beta1::Duration,
        grapl::api::plugin_sdk::generators::v1beta1::{
            client::GeneratorServiceClient,
            server::{GeneratorServer, GeneratorApi},
        },
    },
    protocol::{
        error::ServeError,
        healthcheck::{
            client::HealthcheckClient,
            HealthcheckStatus,
        },
    },
};
use test_context::AsyncTestContext;
use tokio::{
    net::TcpListener,
    task::JoinHandle,
};

pub trait NewGeneratorApi<T> 
where T: GeneratorApi + Send + Sync + 'static,
{
    fn new_generator_api() -> T;
}

pub struct GeneratorTestContext<T> 
where T: GeneratorApi
{
    _api: PhantomData<T>,
    pub client: GeneratorServiceClient,
    server_handle: JoinHandle<Result<(), ServeError>>,
    shutdown_tx: Sender<()>,
}

#[async_trait::async_trait]
impl<T> AsyncTestContext for GeneratorTestContext<T>
where T: GeneratorApi + Send + Sync + 'static,
Self: NewGeneratorApi<T>
{
    async fn setup() -> Self {
        // binding the tcp listener on port 0 tells the operating system to
        // reserve an unused, ephemeral port
        let tcp_listener = TcpListener::bind("0.0.0.0:0")
            .await
            .expect("failed to bind tcp listener");

        // determine the actual port which was bound
        let socket_address = tcp_listener
            .local_addr()
            .expect("failed to obtain socket address");

        // construct an http URI clients can use to connect to server bound to
        // the port.
        let endpoint = format!("http://{}:{}", socket_address.ip(), socket_address.port());

        // TODO: Figure out a way to make this generic!
        let api = Self::new_generator_api();

        let (server, shutdown_tx) = GeneratorServer::new(
            api,
            tcp_listener,
            || async { Ok(HealthcheckStatus::Serving) },
            Duration::from_millis(50),
        );

        let service_name = server.service_name();

        let server_handle = tokio::task::spawn(server.serve());

        HealthcheckClient::wait_until_healthy(
            endpoint.clone(),
            service_name,
            Duration::from_millis(250),
            Duration::from_millis(10),
        )
        .await
        .expect("Generator never reported healthy");

        let client = GeneratorServiceClient::connect(endpoint)
            .await
            .expect("could not configure client");

        GeneratorTestContext {
            _api: PhantomData,
            client,
            server_handle,
            shutdown_tx,
        }
    }

    async fn teardown(self) {
        self.shutdown_tx
            .send(())
            .expect("failed to shutdown server");
        self.server_handle
            .await
            .expect("failed to join server task")
            .expect("server configuration failed");
    }
}
