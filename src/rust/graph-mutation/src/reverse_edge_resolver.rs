use rust_proto::graplinc::grapl::{
    api::graph_schema_manager::v1beta1::{
        client::{
            GraphSchemaManagerClient,
            GraphSchemaManagerClientError,
        },
        messages::{
            GetEdgeSchemaRequest,
            GetEdgeSchemaResponse,
        },
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
    r_edge_cache: dashmap::DashMap<(uuid::Uuid, EdgeName, NodeType), GetEdgeSchemaResponse>,
}

impl ReverseEdgeResolver {
    pub fn new(schema_client: GraphSchemaManagerClient, cache_size: usize) -> Self {
        let r_edge_cache = dashmap::DashMap::with_capacity(cache_size);
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
        let mut schema_client = self.schema_client.clone();
        let response = schema_client
            .get_edge_schema(GetEdgeSchemaRequest {
                tenant_id,
                node_type: node_type.clone(),
                edge_name: edge_name.clone(),
            })
            .await?;

        let reverse_name = response.reverse_edge_name.clone();
        Ok(reverse_name)

        // TODO: Resurrect the below once we figure out caching for Graph Mutation
        // https://github.com/grapl-security/issue-tracker/issues/1028

        // entry.insert(response);
        // match self
        //     .r_edge_cache
        //     .entry((tenant_id, edge_name.clone(), node_type.clone()))
        // {
        //     dashmap::mapref::entry::Entry::Occupied(entry) => {
        //         Ok(entry.get().reverse_edge_name.clone())
        //     }
        //     dashmap::mapref::entry::Entry::Vacant(entry) => {
        //         let mut schema_client = self.schema_client.clone();
        //         let response = schema_client
        //             .get_edge_schema(GetEdgeSchemaRequest {
        //                 tenant_id,
        //                 node_type: node_type.clone(),
        //                 edge_name: edge_name.clone(),
        //             })
        //             .await?;
        //
        //         let reverse_name = response.reverse_edge_name.clone();
        //         entry.insert(response);
        //         Ok(reverse_name)
        //     }
        // }
    }
}
