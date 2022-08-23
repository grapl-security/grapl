use std::net::SocketAddr;

use futures::FutureExt;
use tokio::{
    net::TcpListener,
    sync::oneshot::Receiver,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::{
    graplinc::grapl::api::graph_mutation::v1beta1::messages as native,
    protobufs::graplinc::grapl::api::graph_mutation::v1beta1::{
        self as proto,
        graph_mutation_service_server::{
            GraphMutationService as GraphMutationServiceProto,
            GraphMutationServiceServer as GraphMutationServiceServerProto,
        },
    },
    protocol::{
        healthcheck::{
            server::init_health_service,
            HealthcheckStatus,
        },
        status::Status,
    },
    SerDeError, execute_rpc,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphMutationServiceServerError {
    #[error("grpc transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Bind error: {0}")]
    BindError(std::io::Error),
}

#[tonic::async_trait]
pub trait GraphMutationApi {
    type Error: Into<Status>;
    async fn create_node(
        &self,
        request: native::CreateNodeRequest,
    ) -> Result<native::CreateNodeResponse, Self::Error>;
    async fn set_node_property(
        &self,
        request: native::SetNodePropertyRequest,
    ) -> Result<native::SetNodePropertyResponse, Self::Error>;
    async fn create_edge(
        &self,
        request: native::CreateEdgeRequest,
    ) -> Result<native::CreateEdgeResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T, E> GraphMutationServiceProto for T
where
    T: GraphMutationApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    async fn create_node(
        &self,
        request: tonic::Request<proto::CreateNodeRequest>,
    ) -> Result<tonic::Response<proto::CreateNodeResponse>, tonic::Status> {
        execute_rpc!(self, request, create_node)
    }
    /// SetNodeProperty will update the property of the node with the given uid.
    /// If the node does not exist it will be created.
    async fn set_node_property(
        &self,
        request: tonic::Request<proto::SetNodePropertyRequest>,
    ) -> Result<tonic::Response<proto::SetNodePropertyResponse>, tonic::Status> {
        execute_rpc!(self, request, set_node_property)
    }
    /// CreateEdge will create an edge with the name edge_name between the nodes
    /// that have the given uids. It will also create the reverse edge.
    async fn create_edge(
        &self,
        request: tonic::Request<proto::CreateEdgeRequest>,
    ) -> Result<tonic::Response<proto::CreateEdgeResponse>, tonic::Status> {
        execute_rpc!(self, request, create_edge)
    }
}
