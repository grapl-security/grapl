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
    ) => {{
        {
            // Taking fib sequence 10 times:
            // 1 + 1 + 2 + 3 + 5 + 8 + 13 + 21 + 34 + 55 = 143
            // We're arbitrarily selecting 5000ms as a maximum delay, so we'll
            // choose 5000/143 as our starter, that's 35
            let backoff = client_executor::strategy::FibonacciBackoff::from_millis(35);
            let num_retries = 10;

            let proto_request = <$proto_request_type>::try_from($native_request)?;

            // We can revisit this; potentially passing in a retry_condition
            // per-RPC and not globally applied.
            let retry_condition = |status: &tonic::Status| {
                // Only retry if the status code is Internal Error.
                status.code() == tonic::Code::Internal
            };

            let proto_response = $self
                .executor
                .spawn_conditional(
                    backoff
                        .map(client_executor::strategy::jitter)
                        .take(num_retries),
                    || {
                        let mut proto_client = $self.proto_client.clone();
                        let proto_request = proto_request.clone();
                        async move { proto_client.$rpc_name(proto_request).await }
                    },
                    retry_condition,
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
            // Taking fib sequence 10 times:
            // 1 + 1 + 2 + 3 + 5 + 8 + 13 + 21 + 34 + 55 = 143
            // We're arbitrarily selecting 5000ms as a maximum delay, so we'll
            // choose 5000/143 as our starter, that's 35
            let backoff = client_executor::strategy::FibonacciBackoff::from_millis(35);
            let num_retries = 10;

            let proto_client = $executor
                .spawn(
                    backoff
                        .map(client_executor::strategy::jitter)
                        .take(num_retries),
                    || {
                        let endpoint = $endpoint.clone();
                        async move {
                            <$proto_client_type>::connect(endpoint)
                                .await
                                .map_err(ConnectError::from)
                        }
                    },
                )
                .await?;
            proto_client
        }
    }};
}
