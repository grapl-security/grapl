use std::collections::{HashMap, HashSet};
use std::time::Duration;

use failure::Error;
use graph_descriptions::graph_description::*;
use graph_descriptions::graph_description::id_strategy;
use graph_descriptions::graph_description::Session as SessionStrategy;
use graph_descriptions::node::NodeT;
use rusoto_dynamodb::{
    AttributeValue, DynamoDb, GetItemInput,
    PutItemInput,
};

use crate::assetdb::AssetIdentifier;
use crate::futures::TryFutureExt;
use crate::sessiondb::SessionDb;
use crate::sessions::UnidSession;

#[derive(Debug, Clone)]
pub struct DynamicMappingDb<D>
    where D: DynamoDb
{
    dyn_mapping_db: D
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedMapping {
    pub mapping: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectMapping {
    pub pseudo_key: String,
    pub mapping: String,
}

impl<D> DynamicMappingDb<D> where D: DynamoDb {
    pub fn new(dyn_mapping_db: D) -> Self {
        Self {
            dyn_mapping_db
        }
    }

    pub async fn direct_map(
        &self,
        input: &str,
    ) -> Result<Option<String>, Error> {
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
            table_name: "static_mapping_table".to_owned(),
            key,
            ..Default::default()
        };

        let item = wait_on!(self.dyn_mapping_db.get_item(query))?.item;

        match item {
            Some(item) => {
                let mapping: ResolvedMapping = serde_dynamodb::from_hashmap(item.clone())?;
                Ok(Some(mapping.mapping))
            }
            None => Ok(None)
        }
    }

    pub async fn create_mapping(&self, input: String, maps_to: String) -> Result<(), Error> {
        info!("Creating dynamic mapping for: {} {}", input, maps_to);
        let mapping = DirectMapping {
            pseudo_key: input,
            mapping: maps_to,
        };

        let put_req = PutItemInput {
            item: serde_dynamodb::to_hashmap(&mapping).unwrap(),
            table_name: "static_mapping_table".to_owned(),
            ..Default::default()
        };

        let put_item_response = wait_on!(self.dyn_mapping_db.put_item(put_req))?;

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct DynamicNodeIdentifier<D>
    where D: DynamoDb
{
    asset_identifier: AssetIdentifier<D>,
    dyn_session_db: SessionDb<D>,
    dyn_mapping_db: DynamicMappingDb<D>,
    should_guess: bool,
}

impl<D> DynamicNodeIdentifier<D>
    where D: DynamoDb
{
    pub fn new(
        asset_identifier: AssetIdentifier<D>,
        dyn_session_db: SessionDb<D>,
        dyn_mapping_db: DynamicMappingDb<D>,
        should_guess: bool,
    ) -> Self {
        Self {
            asset_identifier,
            dyn_session_db,
            dyn_mapping_db,
            should_guess,
        }
    }

    async fn primary_session_key(
        &self,
        node: &mut DynamicNode,
        strategy: &SessionStrategy,
    ) -> Result<String, Error> {
        let mut primary_key = String::with_capacity(32);

        if strategy.primary_key_requires_asset_id {
            let asset_id = match node.get_asset_id() {
                Some(asset_id) => asset_id.to_owned(),
                None => {
                    self.asset_identifier.attribute_asset_id(
                        &node.clone().into(),
                    ).await?
                }
            };

            primary_key.push_str(&asset_id);
            node.set_asset_id(asset_id);
        }

        for prop_name in &strategy.primary_key_properties {
            let prop_val = node.properties.get(prop_name);

            match prop_val {
                Some(val) => primary_key.push_str(&val.to_string()),
                None => bail!(
                format!("Node is missing required propery {} for identity", prop_name)
            )
            }
        }

        // Push node type, as a natural partition
        primary_key.push_str(&node.node_key);

        Ok(primary_key)
    }


    async fn primary_mapping_key(&self, node: &mut DynamicNode, strategy: &Static) -> Result<String, Error> {
        let mut primary_key = String::with_capacity(32);

        if strategy.primary_key_requires_asset_id {
            let asset_id = match node.get_asset_id() {
                Some(asset_id) => asset_id.to_owned(),
                None => {
                    self.asset_identifier.attribute_asset_id(
                        &node.clone().into(),
                    ).await?
                }
            };

            primary_key.push_str(&asset_id);
            node.set_asset_id(asset_id);
        }

        for prop_name in &strategy.primary_key_properties {
            let prop_val = node.properties.get(prop_name);

            match prop_val {
                Some(val) => primary_key.push_str(&val.to_string()),
                None => bail!(
                format!("Node is missing required propery {} for identity", prop_name)
            )
            }
        }
        primary_key.push_str(&node.node_type);

        Ok(primary_key)
    }

    pub async fn attribute_dynamic_session(
        &self,
        node: DynamicNode,
        strategy: &SessionStrategy,
    ) -> Result<DynamicNode, Error> {
        let mut attributed_node = node.clone();

        let primary_key = self.primary_session_key(&mut attributed_node, strategy).await?;

        let unid = match (strategy.created_time != 0, strategy.last_seen_time != 0) {
            (true, _) => {
                UnidSession {
                    pseudo_key: primary_key,
                    timestamp: strategy.created_time,
                    is_creation: true,
                }
            }
            (_, true) => {
                UnidSession {
                    pseudo_key: primary_key,
                    timestamp: strategy.last_seen_time,
                    is_creation: false,
                }
            }
            _ => bail!("Terminating sessions not yet supported")
        };

        let session_id = self.dyn_session_db.handle_unid_session(
            unid,
            self.should_guess,
        ).await?;

        attributed_node.set_key(session_id);

        Ok(attributed_node)
    }

    pub async fn attribute_static_mapping(
        &self,
        node: DynamicNode,
        strategy: &Static,
    ) -> Result<DynamicNode, Error> {
        let mut attributed_node = node.clone();
        let key = self.primary_mapping_key(&mut attributed_node, strategy).await?;

        let node_key = self.dyn_mapping_db.direct_map(&key).await?;

        match node_key {
            Some(node_key) => attributed_node.set_key(node_key),
            None => {
                // Static mappings don't need to be guessed, if
                // we don't find it just make it
                let new_id = uuid::Uuid::new_v4().to_string();
                info!("Creating static mapping for dynamic node");
                self.dyn_mapping_db.create_mapping(
                    key,
                    new_id.clone(),
                ).await?;
                attributed_node.set_key(new_id)
            }
        }

        Ok(attributed_node)
    }

    pub async fn attribute_dynamic_node(&self, node: &DynamicNode) -> Result<DynamicNode, Error> {
        let mut attributed_node = node.clone();
        for strategy in node.get_id_strategies() {
            match strategy.strategy.as_ref().unwrap() {
                id_strategy::Strategy::Session(ref strategy) => {
                    info!("Attributing dynamic node via session");
                    attributed_node = self.attribute_dynamic_session(
                        attributed_node,
                        &strategy,
                    ).await?;
                }
                id_strategy::Strategy::Static(ref strategy) => {
                    info!("Attributing dynamic node via static mapping");
                    attributed_node = self.attribute_static_mapping(
                        attributed_node,
                        &strategy,
                    ).await?;
                }
            }
        }

        Ok(attributed_node)
    }

    pub async fn attribute_dynamic_nodes(&self, unid_graph: Graph, unid_id_map: &mut HashMap<String, String>) -> Result<Graph, Graph> {
        let mut unid_id_map = HashMap::new();
        let mut dead_nodes = HashSet::new();
        let mut output_graph = Graph::new(unid_graph.timestamp);
        output_graph.edges = unid_graph.edges;

        for node in unid_graph.nodes.values() {
            let dynamic_node = match node.as_dynamic_node() {
                Some(n) => {
                    n
                }
                _ => {
                    output_graph.add_node(node.clone());
                    continue;
                }
            };

            let new_node = match self.attribute_dynamic_node(&dynamic_node).await {
                Ok(node) => node,
                Err(e) => {
                    warn!("Failed to attribute dynamic node: {}", e);
                    dead_nodes.insert(node.get_node_key());
                    continue;
                }
            };

            info!("Attributed DynamicNode");

            unid_id_map.insert(node.clone_node_key(), new_node.clone_node_key());
            output_graph.add_node(new_node);
        }

        //
        //        remap_nodes(&mut output_graph, &unid_id_map);
        //        remap_edges(&mut output_graph, &unid_id_map);
//        remove_dead_edges(&mut output_graph);

        if dead_nodes.is_empty() {
            info!("Attributed all dynamic nodes");
            Ok(output_graph)
        } else {
            warn!("Failed to attribute {} dynamic nodes", dead_nodes.len());
            Err(output_graph)
        }
    }
}
