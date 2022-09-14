use std::{
    collections::HashMap,
    fmt::Debug,
};

use blake2::{
    digest::consts::U16,
    Blake2b,
    Digest,
};
use rusoto_dynamodb::{
    AttributeValue,
    DynamoDb,
    GetItemInput,
    GetItemOutput,
    PutItemInput,
    PutItemOutput,
};
use rust_proto::graplinc::grapl::{
    api::{
        graph::v1beta1::{
            NodeDescription,
            Static,
        },
        graph_mutation::v1beta1::{
            client::GraphMutationClient,
            messages::CreateNodeRequest,
        },
    },
    common::v1beta1::types::{
        NodeType,
        Uid,
    },
};

type Blake2b16 = Blake2b<U16>;

#[derive(Clone)]
pub struct StaticMappingDb<D> {
    static_mapping_db: D,
    graph_mutation_client: GraphMutationClient,
    table_name: String,
}

impl<D> Debug for StaticMappingDb<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticMappingDb")
            .field("graph_mutation_client", &self.graph_mutation_client)
            .field("table_name", &self.table_name)
            .finish()
    }
}

impl<D> StaticMappingDb<D>
where
    D: DynamoDb,
{
    pub fn new(
        static_mapping_db: D,
        uid_allocator_client: GraphMutationClient,
        table_name: String,
    ) -> Self {
        Self {
            static_mapping_db,
            graph_mutation_client: uid_allocator_client,
            table_name,
        }
    }

    pub async fn map_unid(
        &self,
        tenant_id: uuid::Uuid,
        node: &NodeDescription,
        strategy: &Static,
    ) -> Result<Uid, failure::Error> {
        let static_id = get_static_id(tenant_id, node, strategy)?;

        if let Some(uid) = self.retrieve_uid_from_dynamodb(static_id.clone()).await? {
            return Ok(uid);
        }

        let mut graph_mutation_client = self.graph_mutation_client.clone();
        let uid = graph_mutation_client
            .create_node(CreateNodeRequest {
                tenant_id,
                node_type: NodeType {
                    value: node.node_type.clone(),
                },
            })
            .await?
            .uid;

        // todo: Retry this operation if it fails.
        self.store_uid_in_dynamodb(static_id, uid).await?;

        Ok(uid)
    }

    pub async fn store_uid_in_dynamodb(
        &self,
        static_id: String,
        uid: Uid,
    ) -> Result<(), failure::Error> {
        let key = HashMap::from([
            (
                "static_id".to_string(),
                AttributeValue {
                    s: Some(static_id),
                    ..Default::default()
                },
            ),
            (
                "uid".to_string(),
                AttributeValue {
                    n: Some(uid.as_u64().to_string()),
                    ..Default::default()
                },
            ),
        ]);

        // todo: Consider a `condition_expression` here to ensure the uid is not already set
        //       and if it is, retrieve it from dynamodb instead
        let _response: PutItemOutput = self
            .static_mapping_db
            .put_item(PutItemInput {
                table_name: self.table_name.clone(),
                item: key,
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    pub async fn retrieve_uid_from_dynamodb(
        &self,
        static_id: String,
    ) -> Result<Option<Uid>, failure::Error> {
        let key = HashMap::from([(
            "static_id".to_string(),
            AttributeValue {
                s: Some(static_id),
                ..Default::default()
            },
        )]);
        let response: GetItemOutput = self
            .static_mapping_db
            .get_item(GetItemInput {
                attributes_to_get: Some(vec!["uid".to_string()]),
                consistent_read: Some(true),
                key,
                table_name: self.table_name.clone(),
                ..GetItemInput::default()
            })
            .await?;

        let mut item = match response.item {
            Some(item) => item,
            None => return Ok(None),
        };

        Ok(item
            .remove("uid")
            .and_then(|v| v.n)
            .map(|uid| uid.parse::<u64>().map(|uid| Uid::from_u64(uid).unwrap()))
            .transpose()?)
    }
}

/// Because statically identified nodes are uniquely identifiable based on their static properties
/// we can avoid fetching from dynamodb and calculate a node key by hashing the properties deterministically
#[tracing::instrument(skip(node, strategy), err)]
fn get_static_id(
    tenant_id: uuid::Uuid,
    node: &NodeDescription,
    strategy: &Static,
) -> Result<String, failure::Error> {
    let mut hasher = Blake2b16::new();
    hasher.update(tenant_id.as_bytes());

    // first, let's sort the properties, so we get a consistent ordering for hashing
    let mut sorted_key_properties = strategy.primary_key_properties.clone();
    sorted_key_properties.sort();

    for prop_name in sorted_key_properties {
        match node.properties.get(&prop_name) {
            Some(prop_val) => hasher.update(prop_val.to_string().as_bytes()),
            None => failure::bail!(format!(
                "Node is missing required property {} for identity",
                prop_name
            )),
        }
    }

    hasher.update(node.node_type.as_bytes());

    Ok(hex::encode(hasher.finalize()))
}
