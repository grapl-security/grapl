use moka::future::Cache;
use rust_proto::graplinc::grapl::{
    api::graph_schema_manager::v1beta1::{
        client::{
            GraphSchemaManagerClient,
            GraphSchemaManagerClientError,
        },
        messages::GetEdgeSchemaRequest,
    },
    common::v1beta1::types::{
        EdgeName,
        NodeType,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum ReverseEdgeResolverError {
    #[error("couldn't resolve reverse edge from Graph Schema Manager: {0}")]
    GraphSchemaManagerClientError(#[from] GraphSchemaManagerClientError),
}

#[derive(Clone)]
pub struct ReverseEdgeResolver {
    schema_client: GraphSchemaManagerClient,
    r_edge_cache: Cache<(uuid::Uuid, EdgeName, NodeType), EdgeName>,
}

impl ReverseEdgeResolver {
    pub fn new(schema_client: GraphSchemaManagerClient, cache_size: u64) -> Self {
        let r_edge_cache = Cache::new(cache_size);
        Self {
            schema_client,
            r_edge_cache,
        }
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn resolve_reverse_edge(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        edge_name: EdgeName,
    ) -> Result<EdgeName, ReverseEdgeResolverError> {
        let cache = &self.r_edge_cache;
        let key = (tenant_id, edge_name.clone(), node_type.clone());

        match cache.get(&key) {
            Some(r_edge_name) => Ok(r_edge_name),
            None => {
                let mut schema_client = self.schema_client.clone();
                let response = schema_client
                    .get_edge_schema(GetEdgeSchemaRequest {
                        tenant_id,
                        node_type: node_type.clone(),
                        edge_name: edge_name.clone(),
                    })
                    .await?;

                let r_edge_name = response.reverse_edge_name.clone();
                cache.insert(key, r_edge_name.clone()).await;
                Ok(r_edge_name)
            }
        }
    }
}
