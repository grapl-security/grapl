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
            let proto_request = <$proto_request_type>::try_from($native_request)?;

            let proto_response = $self
                .executor
                .spawn(
                    client_executor::strategy::FibonacciBackoff::from_millis(10)
                        .map(jitter)
                        .take(20),
                    || {
                        let mut proto_client = $self.proto_client.clone();
                        let proto_request = proto_request.clone();
                        async move { proto_client.$rpc_name(proto_request).await }
                    },
                )
                .await?;
            let native_response = <$native_response_type>::try_from(proto_response.into_inner())?;
            Ok(native_response)
        }
    }};
}
