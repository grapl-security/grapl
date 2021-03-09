use tonic::transport::Channel;
use grapl_graph_descriptions::graph_mutation_service::graph_mutation_rpc_client::GraphMutationRpcClient;
use grapl_graph_descriptions::graph_mutation_service::{CreateNodeRequest, CreateNodeSuccess};
use grapl_graph_descriptions::graph_mutation_service::create_node_result;
use tonic::Status;


#[derive(thiserror::Error, Debug)]
pub enum NodeAllocatorError {
    #[error("MutationError")]
    MutationError(#[from] Status)
}

#[derive(Clone, Debug)]
pub struct NodeAllocator {
    pub(crate) mutation_client: GraphMutationRpcClient<Channel>,
}

impl NodeAllocator {
    pub async fn allocate_node(
        &mut self,
        node_type: String,
    ) -> Result<u64, NodeAllocatorError> {
        let res = self.mutation_client
            .create_node(CreateNodeRequest {
                node_type,
            })
            .await?;
        match res
            .into_inner()
            .rpc_result
            .unwrap() {
            create_node_result::RpcResult::Created(CreateNodeSuccess { uid }) => Ok(uid),
            _ => panic!("Failed")
        }
    }
}
