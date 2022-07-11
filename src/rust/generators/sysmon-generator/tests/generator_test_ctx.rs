use futures::channel::oneshot::Sender;
use rust_proto::{
    graplinc::{
        common::v1beta1::Duration,
        grapl::api::plugin_sdk::generators::v1beta1::{
            client::GeneratorServiceClient,
            server::GeneratorServer,
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
use sysmon_generator::api::SysmonGenerator;
use test_context::AsyncTestContext;
use tokio::{
    net::TcpListener,
    task::JoinHandle,
};

pub struct GeneratorTestContext {
    pub client: GeneratorServiceClient,
    server_handle: JoinHandle<Result<(), ServeError>>,
    shutdown_tx: Sender<()>,
}

#[async_trait::async_trait]
impl AsyncTestContext for GeneratorTestContext {
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
        let api = SysmonGenerator {};

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
