/// Return true if this RPC should be retried.
type RetryPredicate = fn(&tonic::Status) -> bool;
pub struct ExecuteClientRpcOptions {
    pub retry_predicate: RetryPredicate,
}

impl Default for ExecuteClientRpcOptions {
    fn default() -> Self {
        let retry_on_nothing = (|_| false) as RetryPredicate;
        Self {
            retry_predicate: retry_on_nothing,
        }
    }
}

/// This macro implements boilerplate code to:
/// - translate between the native types and tonic/prost types
///   in the transport layer
/// - Automatically hook these client RPCs up to the Executor, which provides
///   retries (among other good behaviors).
///
/// It makes some expectations of the structure of `self` - like that it
/// has a `self.executor` and `self.proto_client`.
#[macro_export]
macro_rules! execute_client_rpc {
    (
        $self: ident,
        $native_request: ident,
        $rpc_name: ident,
        $proto_request_type: ty,
        $native_response_type: ty,
        $opts: expr,
    ) => {{
        {
            let backoff = client_executor::strategy::FibonacciBackoff::from_millis(100)
                .max_delay(Duration::from_millis(5000))
                .map(client_executor::strategy::jitter);
            let num_retries = 10;

            let proto_request = <$proto_request_type>::try_from($native_request)?;

            let executor_retry_condition = |status: &tonic::Status| {
                // Always retry if Unavailable, and optionally retry if
                // specified by the ExecuteClientRpcOptions.
                status.code() == tonic::Code::Unavailable || ($opts.retry_predicate)(status)
            };
            let span = tracing::span::Span::current();

            let proto_response = $self
                .executor
                .spawn_conditional(
                    backoff.take(num_retries),
                    || {
                        let mut proto_client = $self.proto_client.clone();
                        let proto_request = proto_request.clone();
                        tracing::Instrument::instrument(
                            async move {
                                let mut tonic_request = tonic::Request::new(proto_request);
                                tonic_request.set_timeout(Duration::from_secs(60));
                                proto_client.$rpc_name(tonic_request).await
                            },
                            span.clone(),
                        )
                    },
                    executor_retry_condition,
                )
                .await?;
            let native_response = <$native_response_type>::try_from(proto_response.into_inner())?;
            Ok(native_response)
        }
    }};
}

/// This macro implements boilerplate code to connect to
/// a gRPC service (and do retries if needed).
/// Unfortunately, each $proto_client_type doesn't share traits, so a
/// macro is the quickest solution.
#[macro_export]
macro_rules! create_proto_client {
    (
        $executor: ident,
        $proto_client_type: ty,
        $endpoint: ident,
    ) => {{
        {
            let backoff = client_executor::strategy::FibonacciBackoff::from_millis(100)
                .max_delay(Duration::from_millis(5000))
                .map(client_executor::strategy::jitter);
            let num_retries = 10;

            let proto_client = $executor
                .spawn(backoff.take(num_retries), || {
                    let endpoint = $endpoint.clone();
                    async move {
                        <$proto_client_type>::connect(endpoint)
                            .await
                            .map_err($crate::protocol::service_client::ConnectError::from)
                    }
                })
                .await?;
            proto_client
        }
    }};
}
