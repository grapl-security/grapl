use std::fmt::Debug;

use bytes::Bytes;
use futures::channel::oneshot::Sender;
use proptest::prelude::*;
use rust_proto_new::{
    graplinc::{
        common::v1beta1::{
            Duration,
            SystemTime,
            Uuid,
        },
        grapl::{
            api::pipeline_ingress::v1beta1::{
                client::{
                    HealthcheckClient,
                    PipelineIngressClient,
                },
                server::{
                    ConfigurationError,
                    PipelineIngressApi,
                    PipelineIngressServer,
                },
                HealthcheckStatus,
                PublishRawLogRequest,
                PublishRawLogResponse,
            },
            pipeline::{
                v1beta1::{
                    Envelope as EnvelopeV1,
                    Metadata,
                    RawLog,
                },
                v1beta2::Envelope,
            },
        },
    },
    SerDe,
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
// ---------------- strategies ------------------------------------------------
//

//
// Bytes
//

fn bytes(size: usize) -> impl Strategy<Value = Bytes> {
    proptest::collection::vec(any::<u8>(), size).prop_map(Bytes::from)
}

//
// Uuid
//

prop_compose! {
    fn uuids()(
        int in any::<u128>()
    ) -> Uuid {
        Uuid::from_u128_le(int)
    }
}

//
// RawLog
//

prop_compose! {
    fn raw_logs()(
        log_event in bytes(256)
    ) -> RawLog {
        RawLog {
            log_event
        }
    }
}

//
// Metadata
//

prop_compose! {
    fn metadatas()(
        tenant_id in uuids(),
        trace_id in uuids(),
        retry_count in any::<u32>(),
        created_time in any::<SystemTime>(),
        last_updated_time in any::<SystemTime>(),
        event_source_id in uuids()
    ) -> Metadata {
        Metadata {
            tenant_id,
            trace_id,
            retry_count,
            created_time,
            last_updated_time,
            event_source_id,
        }
    }
}

//
// Envelope
//

prop_compose! {
    fn v1_envelopes()(
        metadata in metadatas(),
        inner_type in any::<String>(),
        inner_message in bytes(256),
    ) -> EnvelopeV1 {
        EnvelopeV1 {
            metadata,
            inner_type,
            inner_message
        }
    }
}

fn envelopes<T>(inner_strategy: impl Strategy<Value = T>) -> impl Strategy<Value = Envelope<T>>
where
    T: SerDe + Debug,
{
    (metadatas(), inner_strategy).prop_map(|(metadata, inner_message)| -> Envelope<T> {
        Envelope {
            metadata,
            inner_message,
        }
    })
}

//
// PublishRawLogRequest
//

prop_compose! {
    fn publish_raw_log_requests()(
        event_source_id in uuids(),
        tenant_id in uuids(),
        log_event in bytes(256),
    ) -> PublishRawLogRequest {
        PublishRawLogRequest {
            event_source_id,
            tenant_id,
            log_event
        }
    }
}

//
// PublishRawLogResponse
//

prop_compose! {
    fn publish_raw_log_responses()(
        created_time in any::<SystemTime>(),
    ) -> PublishRawLogResponse {
        PublishRawLogResponse {
            created_time,
        }
    }
}

//
// ---------------- helpers ---------------------------------------------------
//

// helper function to define a simple encode-decode invariant
// see: https://hypothesis.works/articles/encode-decode-invariant/
fn check_encode_decode_invariant<T>(serializable: T)
where
    T: SerDe + PartialEq + Clone + Debug,
{
    let cloned = serializable.clone();
    let serialized = serializable.serialize().expect("serialization failed");
    let deserialized = T::deserialize(serialized).expect("deserialization failed");
    assert!(cloned == deserialized);
}

//
// ---------------- protobuf tests ---------------------------------------------
//
// These tests check the encode-decode invariant (and possibly other invariants)
// of the transport objects this crate provides. These tests should use the
// proptest generators and helper functions (defined above) to establish these
// invariants.

proptest! {
    //
    // common
    //

    #[test]
    fn test_duration_encode_decode(duration in any::<Duration>()) {
        check_encode_decode_invariant(duration)
    }

    #[test]
    fn test_system_time_encode_decode(system_time in any::<SystemTime>()) {
        check_encode_decode_invariant(system_time)
    }

    #[test]
    fn test_uuid_encode_decode(uuid in uuids()) {
        check_encode_decode_invariant(uuid)
    }

    //
    // pipeline
    //

    #[test]
    fn test_metadata_encode_decode(metadata in metadatas()) {
        check_encode_decode_invariant(metadata)
    }

    #[test]
    fn test_raw_log_encode_decode(raw_log in raw_logs()) {
        check_encode_decode_invariant(raw_log)
    }

    #[test]
    fn test_v1_envelope_encode_decode(envelope in v1_envelopes()) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_uuid_envelope_encode_decode(envelope in envelopes(uuids())) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_timestamp_envelope_encode_decode(envelope in envelopes(any::<SystemTime>())) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_duration_envelope_encode_decode(envelope in envelopes(any::<Duration>())) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_raw_log_envelope_encode_decode(envelope in envelopes(raw_logs())) {
        check_encode_decode_invariant(envelope)
    }

    //
    // api.pipeline_ingress
    //

    #[test]
    fn test_publish_raw_log_request_encode_decode(
        publish_raw_log_request in publish_raw_log_requests()
    ) {
        check_encode_decode_invariant(publish_raw_log_request)
    }

    #[test]
    fn test_publish_raw_log_response_encode_decode(
        publish_raw_log_response in publish_raw_log_responses()
    ) {
        check_encode_decode_invariant(publish_raw_log_response)
    }

    // TODO: add more here as they're implemented
}

//
// ---------------- gRPC tests -------------------------------------------------
//
// These tests exercise the gRPC machinery. The idea here is to exercise an
// API's success and error paths with a simple mocked business logic
// implementation. The gRPC client should be used to exercise the gRPC
// server.
//

// We can use this global TENANT_ID for two purposes:
//   (1) To make sure some test data made it through to our mocked
//       application layer.
//   (2) To send a poison pill to our mocked application layer, to
//       trigger an error response.
const TENANT_ID: &'static str = "f000b11e-b421-4ffe-87c2-a963b77fd8e9";

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
        let tenant_id = Uuid::parse_str(TENANT_ID).expect("failed to parse tenant_id");
        assert!(request.tenant_id == tenant_id);

        if request.event_source_id == tenant_id {
            // we can trigger the error response by sending a request with
            // event_source_id set to TENANT_ID
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
            Duration::from_millis(10)
        ).await.expect("pipeline-ingress never reported healthy");

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
            tenant_id: Uuid::parse_str(TENANT_ID).expect("failed to parse tenant_id"),
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
            event_source_id: tenant_id,
            tenant_id,
            log_event: "fail!".into(),
        })
        .await
    {
        println!("expected error response, received: {:?}", res);
        panic!("expected error response");
    } else {
        // 👍 great success 👍
    }
}
