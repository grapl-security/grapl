use rust_proto::{
    graplinc::{
        common::v1beta1::Duration,
        grapl::api::plugin_sdk::generators::v1beta1::{
            client::GeneratorServiceClient,
            server::{
                GeneratorApi,
                GeneratorServer,
            },
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
use test_context::{
    futures::channel::oneshot::Sender,
    AsyncTestContext,
};
use tokio::{
    net::TcpListener,
    task::JoinHandle,
};

/*
We have to do this silly Internals song-and-dance for a couple reasons:

- TestContext doesn't really allow parameterization (which GeneratorApi is
  under test?), which I introduce here as .get_client()'s parameter.
  We can pass in any type of concrete GeneratorApi we want, while reusing the
  test_context stuff for any and all Generators.
- We could have skipped using TestContext entirely, and just done roughly
  `let ctx = SomeGeneratorHelper::new(SysmonGenerator{})` - except that the
  `async teardown` is very desirable here. Rust doesn't have AsyncDrop yet.

After experimenting a bit, this seemed the most ergonomic solution.
*/

struct GeneratorTestContextInternals {
    client: GeneratorServiceClient,
    server_handle: JoinHandle<Result<(), ServeError>>,
    shutdown_tx: Sender<()>,
}
impl GeneratorTestContextInternals {
    async fn new(generator_api: impl GeneratorApi + Send + Sync + 'static) -> Self {
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

        let (server, shutdown_tx) = GeneratorServer::new(
            generator_api,
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

        GeneratorTestContextInternals {
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

pub struct GeneratorTestContext {
    internals: Option<GeneratorTestContextInternals>,
}

#[async_trait::async_trait]
impl AsyncTestContext for GeneratorTestContext {
    async fn setup() -> Self {
        Self { internals: None }
    }
    async fn teardown(self) {
        match self.internals {
            Some(i) => i.teardown().await,
            _ => (),
        }
    }
}
impl GeneratorTestContext {
    pub async fn get_client(
        &mut self,
        generator_api: impl GeneratorApi + Send + Sync + 'static,
    ) -> GeneratorServiceClient {
        if let None = self.internals {
            self.internals = Some(GeneratorTestContextInternals::new(generator_api).await)
        }

        self.internals.as_ref().expect("internals").client.clone()
    }
}
