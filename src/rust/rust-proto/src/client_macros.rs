/// Return true if this RPC should be retried.
type RetryPredicate = fn(&tonic::Status) -> bool;
pub struct RpcConfig {
    pub retry_predicate: RetryPredicate,
    pub backoff: RpcBackoffConfig,
}

impl Default for RpcConfig {
    fn default() -> Self {
        let retry_on_nothing = (|_| false) as RetryPredicate;
        Self {
            retry_predicate: retry_on_nothing,
            backoff: RpcBackoffConfig::default(),
        }
    }
}

pub struct RpcBackoffConfig {
    pub initial_millis: u64,
    pub max_delay_millis: u64,
    pub num_retries: usize,
}

impl Default for RpcBackoffConfig {
    /// This would result in the following behavior:
    /// [Try1 -> Wait 100ms] -> [Try2 -> Wait 100ms] -> [Try3 -> Wait 200ms] -> [Try4 -> Wait 300ms] ...
    /// with a maximum of 11 tries (10 retries)
    /// and eventually the wait would max out at 5000ms.
    fn default() -> Self {
        Self {
            initial_millis: 100,
            max_delay_millis: 5000,
            num_retries: 9, // so 10 total tries
        }
    }
}

impl RpcBackoffConfig {
    pub fn num_tries(self) -> usize {
        // Account for the initial try.
        self.num_retries + 1
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
        $rpc_config: expr,
    ) => {{
        {
            // Bind the macro arguments to well typed things.
            let rpc_config: $crate::client_macros::RpcConfig = $rpc_config;
            let backoff_opts: $crate::client_macros::RpcBackoffConfig = rpc_config.backoff;
            let proto_client = $self.proto_client.clone();
            type ProtoRequestType = $proto_request_type;
            type NativeResponseType = $native_response_type;

            // See docs on RpcBackoffConfig::default for a concrete example of this retry backoff timing.
            let backoff = client_executor::strategy::FibonacciBackoff::from_millis(backoff_opts.initial_millis)
                .max_delay(Duration::from_millis(backoff_opts.max_delay_millis))
                .map(client_executor::strategy::jitter);

            // Always retry if Unavailable, and optionally retry if
            // specified by the RpcConfig.
            let executor_retry_condition = |status: &tonic::Status| {
                status.code() == tonic::Code::Unavailable || (rpc_config.retry_predicate)(status)
            };
            let span = tracing::span::Span::current();

            // Convert the Native request - something in `rust-proto` - into a Protobuf request.
            let proto_request: ProtoRequestType = ProtoRequestType::try_from($native_request)?;

            // Execute the RPC, with retries.
            let tonic_response: tonic::Response<_> = $self
                .executor
                .spawn_conditional(
                    backoff.take(backoff_opts.num_tries()),
                    || {
                        let mut proto_client = proto_client.clone();
                        let proto_request = proto_request.clone();
                        tracing::Instrument::instrument(
                            async move {
                                // wrap the protobuf request in a Tonic request
                                let mut tonic_request = tonic::Request::new(proto_request);
                                tonic_request.set_timeout(Duration::from_secs(60));

                                // This is where we actually make the RPC call!
                                // concretely, this maps to, for example,
                                // `self.proto_client.get_generators_for_event_source(tonic_request)`
                                proto_client.$rpc_name(tonic_request).await
                            },
                            span.clone(),
                        )
                    },
                    executor_retry_condition,
                )
                .await?;

            // Unwrap the Tonic Response to get the protobuf response, then convert
            // that into a rust-native response.
            let proto_response = tonic_response.into_inner();
            let native_response: NativeResponseType = NativeResponseType::try_from(proto_response)?;
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
