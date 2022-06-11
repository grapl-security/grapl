use std::{
    collections::HashMap,
    io::Stdout,
};

use grapl_utils::{
    future_ext::GraplFutureExt,
    rusoto_ext::dynamodb::GraplDynamoDbClientExt,
};
use lazy_static::lazy_static;
use rusoto_dynamodb::{
    AttributeValue,
    BatchGetItemInput,
    DynamoDb,
    DynamoDbClient,
    GetItemInput,
    KeysAndAttributes,
};
use rust_proto_new::graplinc::grapl::api::graph::v1beta1::Edge;
use serde::{
    Deserialize,
    Serialize,
};

lazy_static! {
    /// timeout for dynamodb queries
    static ref DYNAMODB_QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);
}

#[derive(Clone)]
pub struct ReverseEdgeResolver {
    dynamo: DynamoDbClient,
    r_edge_cache: std::sync::Arc<std::sync::Mutex<lru::LruCache<String, String>>>,
}

impl ReverseEdgeResolver {
    pub fn new(dynamo: DynamoDbClient, cache_size: usize) -> Self {
        let r_edge_cache = lru::LruCache::new(cache_size);
        let r_edge_cache = std::sync::Arc::new(std::sync::Mutex::new(r_edge_cache));
        Self {
            dynamo,
            r_edge_cache,
        }
    }

    pub async fn resolve_reverse_edges(
        &self,
        edge_name: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let reverse_edge_name = self.resolve_reverse_edges_from_cache(&edge_name).await;

        match reverse_edge_name {
            Some(reverse_edge_name) => Ok(reverse_edge_name),
            None => {
                let r_edge_name = get_r_edge_from_dynamodb(&self.dynamo, edge_name.clone()).await?;

                let cache = self.r_edge_cache.clone();
                let mut cache = cache.lock().unwrap();

                cache.put(edge_name.clone(), r_edge_name.clone());
                cache.put(r_edge_name.clone(), edge_name.clone());
                drop(cache);

                Ok(r_edge_name)
            }
        }
    }

    pub async fn resolve_reverse_edges_from_cache(&self, edge_name: &str) -> Option<String> {
        let cache = self.r_edge_cache.clone();
        let mut cache = cache.lock().unwrap();
        cache.get(edge_name).map(String::from)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EdgeMapping {
    r_edge: String,
}

/// Returns a HashMap of f_edge -> Optional r_edge entries from dynamodb
#[tracing::instrument(skip(client), err)]
pub async fn get_r_edge_from_dynamodb(
    client: &DynamoDbClient,
    f_edge: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let schema_table_name = std::env::var("GRAPL_SCHEMA_TABLE").expect("GRAPL_SCHEMA_TABLE");

    let mut map: HashMap<String, AttributeValue> = HashMap::new();
    map.insert(
        "f_edge".to_owned(),
        AttributeValue {
            s: Some(f_edge),
            ..Default::default()
        },
    );

    let edge_schema = client
        .get_item(GetItemInput {
            consistent_read: Some(false),
            key: map,
            table_name: schema_table_name,
            ..Default::default()
        })
        .await?
        .item;

    let edge_schema = match edge_schema {
        Some(edge_schema) => edge_schema,
        None => {
            tracing::error!(message = "Reverse mapping for edge not found",);
            todo!("Error handling");
        }
    };

    tracing::debug!(message = "Got reverse edge mapping from DynamoDB database.",);

    let edge_mapping = serde_dynamodb::from_hashmap::<EdgeMapping, _>(edge_schema)?;

    Ok(edge_mapping.r_edge)
}
