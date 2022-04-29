use futures::channel::oneshot::Sender;
use rust_proto_new::{
    graplinc::{
        common::v1beta1::{
            Duration,
            Uuid,
        },
        grapl::api::pipeline_ingress::v1beta1::{
            client::PipelineIngressClient,
            server::{
                ConfigurationError,
                PipelineIngressApi,
                PipelineIngressServer,
            },
            PublishRawLogRequest,
            PublishRawLogResponse,
        },
    },
    protocol::healthcheck::{
        client::HealthcheckClient,
        HealthcheckStatus,
    },
};
use test_context::{
    test_context,
    AsyncTestContext,
};
use thiserror::Error;
use tokio::{
    net::TcpListener,
    task::JoinHandle,
};

//
// ---------------- gRPC tests -------------------------------------------------
//
// These tests exercise the gRPC machinery. The idea here is to exercise an
// API's success and error paths with a simple mocked business logic
// implementation. The gRPC client should be used to exercise the gRPC
// server.
//

// This global TENANT_ID is used to check that some test data made it through to
// our mocked application layer
const TENANT_ID: &'static str = "f000b11e-b421-4ffe-87c2-a963b77fd8e9";

// This global BAD_EVENT_SOURCE_ID is used as a poison pill to trigger the error
// response in our mocked application layer
const BAD_EVENT_SOURCE_ID: &'static str = "762d1d31-19c9-4fa5-9eee-91818997adba";

//
// api.pipeline_ingress
//
// These tests exercise the transport layer of the pipeline ingress API. The
// application layer (e.g. business logic) is injected as a dependency, so we
// test that business logic where it's defined (e.g. in the pipeline ingress
// service).

// first we implement a simple mock of the service's business logic. We'll use
// this to exercise the happy path (returning an OK response) and the error path
// (returning an Err response).

struct MockPipelineIngressApi {}

#[derive(Debug, Error)]
enum MockPipelineIngressApiError {
    #[error("failed to publish raw log")]
    PublishRawLogFailed,
}

#[tonic::async_trait]
impl PipelineIngressApi<MockPipelineIngressApiError> for MockPipelineIngressApi {
    async fn publish_raw_log(
        &self,
        request: PublishRawLogRequest,
    ) -> Result<PublishRawLogResponse, MockPipelineIngressApiError> {
        let tenant_id = Uuid::parse_str(TENANT_ID).expect("failed to parse TENANT_ID");
        assert!(request.tenant_id == tenant_id);

        let bad_event_source_id =
            Uuid::parse_str(BAD_EVENT_SOURCE_ID).expect("failed to parse BAD_EVENT_SOURCE_ID");

        if request.event_source_id == bad_event_source_id {
            // we can trigger the error response by sending a request with
            // event_source_id set to BAD_EVENT_SOURCE_ID
            Err(MockPipelineIngressApiError::PublishRawLogFailed)
        } else {
            // otherwise send a success response
            Ok(PublishRawLogResponse::ok())
        }
    }
}

struct PipelineIngressTestContext {
    client: PipelineIngressClient,
    server_handle: JoinHandle<Result<(), ConfigurationError>>,
    shutdown_tx: Sender<()>,
}

#[tonic::async_trait]
impl AsyncTestContext for PipelineIngressTestContext {
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

        let (server, shutdown_tx) = PipelineIngressServer::new(
            MockPipelineIngressApi {},
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
        .expect("pipeline-ingress never reported healthy");

        let client = PipelineIngressClient::connect(endpoint)
            .await
            .expect("could not configure client");

        PipelineIngressTestContext {
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

#[test_context(PipelineIngressTestContext)]
#[tokio::test]
async fn test_publish_raw_log_returns_ok_response(ctx: &mut PipelineIngressTestContext) {
    ctx.client
        .publish_raw_log(PublishRawLogRequest {
            event_source_id: Uuid::new_v4(),
            tenant_id: Uuid::parse_str(TENANT_ID).expect("failed to parse TENANT_ID"),
            log_event: "success!".into(),
        })
        .await
        .expect("received error response");
}

#[test_context(PipelineIngressTestContext)]
#[tokio::test]
async fn test_publish_raw_log_returns_err_response(ctx: &mut PipelineIngressTestContext) {
    let tenant_id = Uuid::parse_str(TENANT_ID).expect("failed to parse tenant_id");

    if let Ok(res) = ctx
        .client
        .publish_raw_log(PublishRawLogRequest {
            event_source_id: Uuid::parse_str(BAD_EVENT_SOURCE_ID)
                .expect("failed to parse BAD_EVENT_SOURCE_ID"),
            tenant_id,
            log_event: "fail!".into(),
        })
        .await
    {
        tracing::error!(
            message = "expected error response",
            response = ?res,
        );
        panic!("expected error response");
    } else {
        // 👍 great success 👍
    }
}
