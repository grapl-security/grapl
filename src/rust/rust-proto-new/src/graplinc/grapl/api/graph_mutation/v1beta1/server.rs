#![allow(warnings)]

use crate::protobufs::graplinc::grapl::api::graph_mutation::v1beta1::{
    CreateNodeRequest as CreateNodeRequestProto,
    CreateNodeResponse as CreateNodeResponseProto,
    SetNodePropertyRequest as SetNodePropertyRequestProto,
    SetNodePropertyResponse as SetNodePropertyResponseProto,
    CreateEdgeResponse as CreateEdgeResponseProto,
    CreateEdgeRequest as CreateEdgeRequestProto,
};
use crate::protobufs::graplinc::grapl::api::graph_mutation::v1beta1::graph_mutation_service_server::{
    GraphMutationService as GraphMutationServiceProto,
    GraphMutationServiceServer as GraphMutationServiceServerProto,
};
use crate::graplinc::grapl::api::graph_mutation::v1beta1::messages::{CreateNodeRequest, CreateNodeResponse, SetNodePropertyRequest, SetNodePropertyResponse, CreateEdgeResponse, CreateEdgeRequest};
use std::net::SocketAddr;

use tonic::{
    transport::Server,
    Request,
    Response,
};

// temporarily export Status
pub use tonic::Status;

use crate::SerDeError;


#[tonic::async_trait]
pub trait GraphMutationApi {
    type Error: Into<Status>;
    // todo: swap out for rust-proto-new::Status when it's available
    async fn create_node(&self, request: CreateNodeRequest) -> Result<CreateNodeResponse, Self::Error>;
    async fn set_node_property(&self, request: SetNodePropertyRequest) -> Result<SetNodePropertyResponse, Self::Error>;
    async fn create_edge(&self, request: CreateEdgeRequest) -> Result<CreateEdgeResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T, E> GraphMutationServiceProto for T
    where T: GraphMutationApi<Error=E> + Send + Sync + 'static,
          E: Into<Status> + Send + Sync + 'static,
{
    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    async fn create_node(
        &self,
        request: tonic::Request<CreateNodeRequestProto>,
    ) -> Result<tonic::Response<CreateNodeResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request.try_into().map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = GraphMutationApi::create_node(self, request).await.map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }
    /// SetNodeProperty will update the property of the node with the given uid.
    /// If the node does not exist it will be created.
    async fn set_node_property(
        &self,
        request: tonic::Request<SetNodePropertyRequestProto>,
    ) -> Result<tonic::Response<SetNodePropertyResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request.try_into().map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = GraphMutationApi::set_node_property(self, request).await.map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }
    /// CreateEdge will create an edge with the name edge_name between the nodes
    /// that have the given uids. It will also create the reverse edge.
    async fn create_edge(
        &self,
        request: tonic::Request<CreateEdgeRequestProto>,
    ) -> Result<tonic::Response<CreateEdgeResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request: CreateEdgeRequest = request.try_into().map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = GraphMutationApi::create_edge(self, request).await.map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }
}