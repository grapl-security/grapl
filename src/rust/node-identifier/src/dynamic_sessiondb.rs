use std::collections::{HashMap};

use failure::{bail,
              Error};
use grapl_graph_descriptions::graph_description::{id_strategy,
                                                  Session as SessionStrategy,
                                                  *};
use log::{info};
use rusoto_dynamodb::{AttributeValue,
                      DynamoDb,
                      GetItemInput,
                      PutItemInput};
use serde::{Deserialize,
            Serialize};

use crate::{sessiondb::SessionDb,
            sessions::UnidSession};
use crate::node_allocator::NodeAllocator;

#[derive(Debug, Clone)]
pub struct DynamicMappingDb<D>
    where
        D: DynamoDb,
{
    dyn_mapping_db: D,
    node_allocator: NodeAllocator,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedMapping {
    pub mapping: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectMapping {
    pub pseudo_key: String,
    pub mapping: u64,
}

impl<D> DynamicMappingDb<D>
    where
        D: DynamoDb,
{
    pub fn new(dyn_mapping_db: D, node_allocator: NodeAllocator) -> Self {
        Self { dyn_mapping_db, node_allocator }
    }

    pub async fn direct_map(&self, input: &str) -> Result<Option<u64>, Error> {
        let mut key: HashMap<String, AttributeValue> = HashMap::new();

        key.insert(
            "pseudo_key".to_owned(),
            AttributeValue {
                s: Some(input.to_owned()),
                ..Default::default()
            },
        );

        let query = GetItemInput {
            consistent_read: Some(true),
            table_name: grapl_config::static_mapping_table_name(),
            key,
            ..Default::default()
        };

        let item = wait_on!(self.dyn_mapping_db.get_item(query))?.item;

        match item {
            Some(item) => {
                let mapping: ResolvedMapping = serde_dynamodb::from_hashmap(item.clone())?;
                Ok(Some(mapping.mapping))
            }
            None => Ok(None),
        }
    }

    pub async fn create_mapping(&self, input: String, maps_to: u64) -> Result<(), Error> {
        info!("Creating dynamic mapping for: {} {}", input, maps_to);
        let mapping = DirectMapping {
            pseudo_key: input,
            mapping: maps_to,
        };

        let put_req = PutItemInput {
            item: serde_dynamodb::to_hashmap(&mapping).unwrap(),
            table_name: grapl_config::static_mapping_table_name(),
            ..Default::default()
        };

        let _put_item_response = wait_on!(self.dyn_mapping_db.put_item(put_req))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NodeDescriptionIdentifier<D>
    where
        D: DynamoDb,
{
    dyn_session_db: SessionDb<D>,
    dyn_mapping_db: DynamicMappingDb<D>,
    should_guess: bool,
}

impl<D> NodeDescriptionIdentifier<D>
    where
        D: DynamoDb,
{
    pub fn new(
        dyn_session_db: SessionDb<D>,
        dyn_mapping_db: DynamicMappingDb<D>,
        should_guess: bool,
    ) -> Self {
        Self {
            dyn_session_db,
            dyn_mapping_db,
            should_guess,
        }
    }

    async fn primary_session_key(
        &self,
        node: &mut NodeDescription,
        strategy: &SessionStrategy,
    ) -> Result<String, Error> {
        let mut primary_key = String::with_capacity(32);

        if strategy.primary_key_requires_asset_id {
            panic!("asset_id resolution is currently not supported")
        }

        for prop_name in &strategy.primary_key_properties {
            let prop_val = node.properties.get(prop_name);

            match prop_val {
                Some(val) => primary_key.push_str(&val.to_string()),
                None => bail!(format!(
                    "Node is missing required property {} for identity",
                    prop_name
                )),
            }
        }

        // Push node type, as a natural partition
        primary_key.push_str(&node.node_key);

        Ok(primary_key)
    }

    async fn primary_mapping_key(
        &self,
        node: &NodeDescription,
        strategy: &Static,
    ) -> Result<String, Error> {
        let mut primary_key = String::with_capacity(32);

        if strategy.primary_key_requires_asset_id {
            panic!("pre-resolution of asset_id is not supported currently")
        }

        for prop_name in &strategy.primary_key_properties {
            let prop_val = node.properties.get(prop_name);

            match prop_val {
                Some(val) => primary_key.push_str(&val.to_string()),
                None => bail!(format!(
                    "Node is missing required property {} for identity",
                    prop_name
                )),
            }
        }
        primary_key.push_str(&node.node_type);

        Ok(primary_key)
    }

    pub async fn attribute_dynamic_session(
        &self,
        node: NodeDescription,
        strategy: &SessionStrategy,
    ) -> Result<IdentifiedNode, Error> {
        let mut attributed_node = node.clone();

        let primary_key = self
            .primary_session_key(&mut attributed_node, strategy)
            .await?;

        let created_time = strategy.create_time;
        let last_seen_time = strategy.last_seen_time;
        let unid = match (created_time != 0, last_seen_time != 0) {
            (true, _) => UnidSession {
                pseudo_key: primary_key,
                node_type: node.node_type,
                timestamp: created_time,
                is_creation: true,
            },
            (_, true) => UnidSession {
                pseudo_key: primary_key,
                node_type: node.node_type,
                timestamp: last_seen_time,
                is_creation: false,
            },
            _ => bail!(
                "Terminating sessions not yet supported: {:?} {:?}",
                node.properties,
                &strategy,
            ),
        };

        let session_id = self
            .dyn_session_db
            .handle_unid_session(unid, self.should_guess)
            .await?;

        Ok(IdentifiedNode {
            uid: session_id,
            node_type: attributed_node.node_type,
            properties: attributed_node.properties,
        })
    }

    pub async fn attribute_static_mapping(
        &self,
        node: NodeDescription,
        strategy: &Static,
    ) -> Result<IdentifiedNode, Error> {
        let key = self
            .primary_mapping_key(&node, strategy)
            .await?;

        let uid = self.dyn_mapping_db.direct_map(&key).await?;

        let uid = match uid {
            Some(uid) => uid,
            None => {
                // Static mappings don't need to be guessed, if
                // we don't find it just make it
                let new_id = self.dyn_mapping_db.node_allocator.clone().allocate_node(node.node_type.clone()).await?;
                info!("Creating static mapping for dynamic node");
                self.dyn_mapping_db
                    .create_mapping(key, new_id)
                    .await?;
                new_id
            }
        };

        Ok(IdentifiedNode {
            uid,
            node_type: node.node_type,
            properties: node.properties,
        })
    }

    pub async fn attribute_dynamic_node(
        &self,
        node: &NodeDescription,
    ) -> Result<IdentifiedNode, Error> {
        let strategy = &node.id_strategy[0];

        match strategy.strategy.as_ref().unwrap() {
            id_strategy::Strategy::Session(ref strategy) => {
                info!("Attributing dynamic node via session");
                Ok(self
                    .attribute_dynamic_session(node.clone(), &strategy)
                    .await?)
            }
            id_strategy::Strategy::Static(ref strategy) => {
                info!("Attributing dynamic node via static mapping");
                Ok(self
                    .attribute_static_mapping(node.clone(), &strategy)
                    .await?)
            }
        }
    }
}
