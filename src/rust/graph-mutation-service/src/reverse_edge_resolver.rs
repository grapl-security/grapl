use std::{
    collections::HashMap,
    io::Stdout,
};

use grapl_utils::{
    future_ext::GraplFutureExt,
    rusoto_ext::dynamodb::GraplDynamoDbClientExt,
};
use lazy_static::lazy_static;
use rust_proto_new::graplinc::grapl::api::schema_manager::v1beta1::client::SchemaManagerClient;
use rust_proto_new::graplinc::grapl::api::schema_manager::v1beta1::messages::{GetEdgeSchemaRequest, GetEdgeSchemaResponse};

#[derive(Clone)]
pub struct ReverseEdgeResolver {
    schema_client: SchemaManagerClient,
    r_edge_cache: dashmap::DashMap<(uuid::Uuid, String, String), GetEdgeSchemaResponse>,
}

impl ReverseEdgeResolver {
    pub fn new(schema_client: SchemaManagerClient, cache_size: usize) -> Self {
        let r_edge_cache = dashmap::DashMap::with_capacity(cache_size);
        Self {
            schema_client,
            r_edge_cache,
        }
    }

    pub async fn resolve_reverse_edge(
        &self,
        tenant_id: uuid::Uuid,
        node_type: String,
        edge_name: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self.r_edge_cache.entry((tenant_id, node_type.clone(), edge_name.clone())) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                Ok(entry.get().reverse_edge_name.clone())
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let mut schema_client = self.schema_client.clone();
                let response = schema_client.get_edge_schema(
                    GetEdgeSchemaRequest {
                        tenant_id,
                        node_type: node_type.clone(),
                        edge_name: edge_name.clone(),
                    },
                ).await?;

                let reverse_name = response.reverse_edge_name.clone();
                entry.insert(response);
                Ok(reverse_name)
            }
        }
    }
}
