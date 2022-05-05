use std::marker::PhantomData;

use crate::protocol::status::Status;

pub trait Api {}

/// This struct implements the internal gRPC representation of the server.
/// We've implemented the service trait generated by tonic
/// in such a way that it delegates to an externally supplied
/// Api. This way all the protocol buffer compiler generated
/// types are encapsulated, and the public API is implemented in terms of
/// this crate's sanitized types.
pub struct ApiDelegate<T, E>
where
    E: Into<Status>,
{
    pub api_server: T,
    _e: PhantomData<E>,
}

impl<T, E> ApiDelegate<T, E>
where
    E: Into<Status>,
{
    pub fn new(api_server: T) -> Self {
        ApiDelegate {
            api_server,
            _e: PhantomData,
        }
    }
}

impl From<crate::SerDeError> for tonic::Status {
    fn from(e: crate::SerDeError) -> Self {
        tonic::Status::unknown(e.to_string())
    }
}

/// Deduplicates the translation layer code in our Server rpc functions.
/// - turn a proto-request into a native-request
/// - feed that into our api server to get a native-response
/// - turn native-response into Response(proto-response)
/// Ideally this would be a generic function, but rust has issues with
/// async function pointers (like self.api_server.any_rpc).
#[macro_export]
macro_rules! rpc_translate_proto_to_native {
    ($self: ident, $request: ident, $rpc_name: ident) => {{
        {
            let proto_request = $request.into_inner();

            let native_request = proto_request.try_into()?;

            let native_response = $self
                .api_server
                .$rpc_name(native_request)
                .await
                .map_err(crate::protocol::status::Status::from)?;

            let proto_response = native_response.try_into().map_err(SerDeError::from)?;

            Ok(tonic::Response::new(proto_response))
        }
    }};
}
