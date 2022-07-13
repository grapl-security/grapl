#![allow(warnings)]

use rust_proto::graplinc::grapl::api::graph_mutation::v1beta1::client::{GraphMutationClient, GraphMutationClientError};
use rust_proto::graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::messages::ExecutionHit;

#[derive(thiserror::Error, Debug)]
pub enum EngagementCreatorError {
    #[error("GraphMutationClientError {0}")]
    GraphMutationClientError(#[from] GraphMutationClientError),
}

#[derive(Clone)]
pub struct EngagementCreator {
    graph_mutation_client: GraphMutationClient,
}

impl EngagementCreator {
    pub fn new(graph_mutation_client: GraphMutationClient) -> Self {
        EngagementCreator {
            graph_mutation_client,
        }
    }
}

impl EngagementCreator {
    pub async fn handle_event(
        &self,
        tenant_id: uuid::Uuid,
        execution_hit: ExecutionHit,
    ) -> Result<(), EngagementCreatorError> {

        Ok(())
    }
}