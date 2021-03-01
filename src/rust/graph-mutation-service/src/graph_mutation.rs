#[derive(Eq, Clone, PartialEq, ::prost::Message)]
pub struct Ok {}
#[derive(Eq, Clone, PartialEq, ::prost::Message)]
pub struct Err {}
#[derive(Eq, Clone, PartialEq, ::prost::Message)]
pub struct SetNodeRequest {
    #[prost(message, optional, tag = "1")]
    pub node: ::core::option::Option<super::v1beta1::IdentifiedNode>,
}
#[derive(Eq, Clone, PartialEq, ::prost::Message)]
pub struct SetNodeResult {
    #[prost(oneof = "set_node_result::RpcResult", tags = "1, 2, 3, 4")]
    pub rpc_result: ::core::option::Option<set_node_result::RpcResult>,
}
/// Nested message and enum types in `SetNodeResult`.
pub mod set_node_result {
    #[derive(Eq, Clone, PartialEq, ::prost::Oneof)]
    pub enum RpcResult {
        #[prost(message, tag = "1")]
        Set(super::Ok),
        #[prost(message, tag = "2")]
        AlreadySet(super::Ok),
        #[prost(message, tag = "3")]
        PredicateDoesNotExist(super::Err),
        #[prost(message, tag = "4")]
        PredicateTypeMismatch(super::Err),
    }
}
#[derive(Eq, Clone, PartialEq, ::prost::Message)]
pub struct SetNodeResponse {
    #[prost(message, optional, tag = "1")]
    pub rpc_result: ::core::option::Option<SetNodeResult>,
}
#[derive(Eq, Clone, PartialEq, ::prost::Message)]
pub struct SetEdgeRequest {
    #[prost(string, tag = "1")]
    pub from_node_key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub from_node_type: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub to_node_key: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub to_node_type: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub f_edge_name: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub r_edge_name: ::prost::alloc::string::String,
}
#[derive(Eq, Clone, PartialEq, ::prost::Message)]
pub struct SetEdgeResult {
    #[prost(oneof = "set_edge_result::RpcResult", tags = "1, 2, 3, 4")]
    pub rpc_result: ::core::option::Option<set_edge_result::RpcResult>,
}
/// Nested message and enum types in `SetEdgeResult`.
pub mod set_edge_result {
    #[derive(Eq, Clone, PartialEq, ::prost::Oneof)]
    pub enum RpcResult {
        #[prost(message, tag = "1")]
        Set(super::Ok),
        #[prost(message, tag = "2")]
        Negated(super::Ok),
        #[prost(message, tag = "3")]
        EdgeDoesNotExist(super::Err),
        #[prost(message, tag = "4")]
        EdgeTypeMismatch(super::Err),
    }
}
#[derive(Eq, Clone, PartialEq, ::prost::Message)]
pub struct SetEdgeResponse {
    #[prost(message, optional, tag = "1")]
    pub rpc_result: ::core::option::Option<SetEdgeResult>,
}
#[doc = r" Generated client implementations."]
pub mod graph_mutation_rpc_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct GraphMutationRpcClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GraphMutationRpcClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> GraphMutationRpcClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        pub async fn set_node(
            &mut self,
            request: impl tonic::IntoRequest<super::SetNodeRequest>,
        ) -> Result<tonic::Response<super::SetNodeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/graplinc.grapl.api.graph.graph_mutation.GraphMutationRpc/SetNode",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn set_edge(
            &mut self,
            request: impl tonic::IntoRequest<super::SetEdgeRequest>,
        ) -> Result<tonic::Response<super::SetEdgeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/graplinc.grapl.api.graph.graph_mutation.GraphMutationRpc/SetEdge",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for GraphMutationRpcClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for GraphMutationRpcClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "GraphMutationRpcClient {{ ... }}")
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod graph_mutation_rpc_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with GraphMutationRpcServer."]
    #[async_trait]
    pub trait GraphMutationRpc: Send + Sync + 'static {
        async fn set_node(
            &self,
            request: tonic::Request<super::SetNodeRequest>,
        ) -> Result<tonic::Response<super::SetNodeResponse>, tonic::Status>;
        async fn set_edge(
            &self,
            request: tonic::Request<super::SetEdgeRequest>,
        ) -> Result<tonic::Response<super::SetEdgeResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct GraphMutationRpcServer<T: GraphMutationRpc> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: GraphMutationRpc> GraphMutationRpcServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, None);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, Some(interceptor.into()));
            Self { inner }
        }
    }
    impl<T, B> Service<http::Request<B>> for GraphMutationRpcServer<T>
    where
        T: GraphMutationRpc,
        B: HttpBody + Send + Sync + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/graplinc.grapl.api.graph.graph_mutation.GraphMutationRpc/SetNode" => {
                    #[allow(non_camel_case_types)]
                    struct SetNodeSvc<T: GraphMutationRpc>(pub Arc<T>);
                    impl<T: GraphMutationRpc> tonic::server::UnaryService<super::SetNodeRequest> for SetNodeSvc<T> {
                        type Response = super::SetNodeResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetNodeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).set_node(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = SetNodeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/graplinc.grapl.api.graph.graph_mutation.GraphMutationRpc/SetEdge" => {
                    #[allow(non_camel_case_types)]
                    struct SetEdgeSvc<T: GraphMutationRpc>(pub Arc<T>);
                    impl<T: GraphMutationRpc> tonic::server::UnaryService<super::SetEdgeRequest> for SetEdgeSvc<T> {
                        type Response = super::SetEdgeResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetEdgeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).set_edge(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = SetEdgeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: GraphMutationRpc> Clone for GraphMutationRpcServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: GraphMutationRpc> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: GraphMutationRpc> tonic::transport::NamedService for GraphMutationRpcServer<T> {
        const NAME: &'static str = "graplinc.grapl.api.graph.graph_mutation.GraphMutationRpc";
    }
}
