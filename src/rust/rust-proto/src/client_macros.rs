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
            let backoff = client_executor::strategy::FibonacciBackoff::from_millis(100)
                .max_delay(Duration::from_millis(5000))
                .map(client_executor::strategy::jitter);
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
                    backoff.take(num_retries),
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
                            .map_err(ConnectError::from)
                    }
                })
                .await?;
            proto_client
        }
    }};
}
