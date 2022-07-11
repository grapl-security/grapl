#![allow(warnings)]
use rust_proto_new::graplinc::grapl::api::graph_mutation::v1beta1::client::GraphMutationClient;
use rust_proto_new::graplinc::grapl::api::graph_mutation::v1beta1::messages::{CreateEdgeRequest, CreateNodeRequest};
use rust_proto_new::graplinc::grapl::api::lens_manager::v1beta1::messages::{AddNodeToScopeRequest, AddNodeToScopeResponse, CloseLensRequest, CloseLensResponse, CreateLensRequest, CreateLensResponse, MergeLensRequest, MergeLensResponse, RemoveNodeFromAllScopesRequest, RemoveNodeFromAllScopesResponse, RemoveNodeFromScopeRequest, RemoveNodeFromScopeResponse};
use rust_proto_new::graplinc::grapl::api::lens_manager::v1beta1::server::server::LensManagerApi;
use rust_proto_new::graplinc::grapl::common::v1beta1::types::{EdgeName, NodeType, Uid};
use rust_proto_new::protocol::status::Status;


#[derive(thiserror::Error, Debug)]
pub enum LensManagerServerError {
    #[error("todo!")]
    Unknown
}

impl From<LensManagerServerError> for Status {
    fn from(_: LensManagerServerError) -> Self {
        todo!()
    }
}

pub struct LensManager {
    graph_mutation_client: GraphMutationClient
}


#[async_trait::async_trait]
impl LensManagerApi for LensManager {
    type Error = LensManagerServerError;

    async fn create_lens(&self, request: CreateLensRequest) -> Result<CreateLensResponse, Status> {
        let mut client = self.graph_mutation_client.clone();

        let create_request = CreateNodeRequest {
            tenant_id: request.tenant_id,
            node_type: NodeType {value: "Lens".to_owned()},
        };

        let response = client.create_node(create_request).await?;


        Ok(CreateLensResponse{
            lens_uid: response.uid.as_u64(),
        })
    }

    async fn merge_lens(&self, request: MergeLensRequest) -> Result<MergeLensResponse, Status> {
        let mut client = self.graph_mutation_client.clone();

        let create_request = MergeLensRequest {
            tenant_id: request.tenant_id,
            source_lens_uid: request.source_lens_uid,
            target_lens_uid: request.target_lens_uid,
            merge_behavior: request.merge_behavior,
        };

        client.merge_lens(create_request).await?;

        Ok(MergeLensResponse{})

    }

    async fn close_lens(&self, request: CloseLensRequest) -> Result<CloseLensResponse, Status> {
        let mut client = self.graph_mutation_client.clone();

        let create_request = CloseLensRequest {
            tenant_id: request.tenant_id,
            lens_uid: request.lens_uid,
        };

        client.close_lens(create_request).await?;

        Ok(CloseLensResponse{})
    }

    async fn add_node_to_scope(&self, request: AddNodeToScopeRequest) -> Result<AddNodeToScopeResponse, Status> {

        let mut client = self.graph_mutation_client.clone();

        let create_request = CreateEdgeRequest {
            edge_name: EdgeName {value: "scope".to_owned()},
            tenant_id: request.tenant_id,
            from_uid: Uid::from_u64(request.lens_uid).unwrap(),
            to_uid: Uid::from_u64(request.uid).unwrap(),
            source_node_type: NodeType {value: "Lens".to_owned()},
            dest_node_type: request.node_type
        };

        client.create_edge(create_request).await?;

        Ok(AddNodeToScopeResponse{})
    }

    async fn remove_node_from_scope(&self, request: RemoveNodeFromScopeRequest) -> Result<RemoveNodeFromScopeResponse, Status> {
        let mut client = self.graph_mutation_client.clone();

        let create_request = AddNodeToScopeRequest {
            tenant_id: request.tenant_id,
            lens_uid: request.lens_uid,
            uid: request.uid,
            node_type: NodeType {value: "Lens".to_owned()},
        };

        client.remove_node_from_scope(create_request).await?;

        Ok(RemoveNodeFromScopeResponse{})
    }

    async fn remove_node_from_all_scopes(&self, request: RemoveNodeFromAllScopesRequest) -> Result<RemoveNodeFromAllScopesResponse, Status> {
        let mut client = self.graph_mutation_client.clone();

        let create_request = RemoveNodeFromScopeRequest {
            tenant_id: request.tenant_id,
            lens_uid: request.uid, // RemoveNodeFromAllScopesRequest does not have lens_uid field - do we need one?
            uid: request.uid
        };

        client.remove_node_from_scope(create_request).await?;

        Ok(RemoveNodeFromAllScopesResponse{})
    }
}