use std::collections::{
    HashMap,
    HashSet,
};

use failure::{
    bail,
    Error,
};
use rusoto_dynamodb::DynamoDb;
use rust_proto::graplinc::grapl::api::graph::v1beta1::{
    GraphDescription,
    NodeDescription,
    Session,
    Static,
    Strategy,
};
use serde::{
    Deserialize,
    Serialize,
};
use sha2::{
    Digest,
    Sha256,
};

use crate::{
    sessiondb::SessionDb,
    sessions::UnidSession,
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ResolvedMapping {
    pub mapping: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DirectMapping {
    pub pseudo_key: String,
    pub mapping: String,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeDescriptionIdentifier<D>
where
    D: DynamoDb,
{
    dyn_session_db: SessionDb<D>,
    should_guess: bool,
}

impl<D> NodeDescriptionIdentifier<D>
where
    D: DynamoDb,
{
    pub fn new(dyn_session_db: SessionDb<D>, should_guess: bool) -> Self {
        Self {
            dyn_session_db,
            should_guess,
        }
    }

    #[tracing::instrument(skip(self, node, strategy), err)]
    async fn primary_session_key(
        &self,
        node: &mut NodeDescription,
        strategy: &Session,
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

    /// Because statically identified nodes are uniquely identifiable based on their static properties
    /// we can avoid fetching from dynamodb and calculate a node key by hashing the properties deterministically
    #[tracing::instrument(skip(self, node, strategy), err)]
    fn get_static_node_key(
        &self,
        node: &NodeDescription,
        strategy: &Static,
    ) -> Result<String, Error> {
        let mut hasher = Sha256::new();

        // first, let's sort the properties, so we get a consistent ordering for hashing
        let mut sorted_key_properties = strategy.primary_key_properties.clone();
        sorted_key_properties.sort();

        for prop_name in sorted_key_properties {
            match node.properties.get(&prop_name) {
                Some(prop_val) => hasher.update(prop_val.to_string().as_bytes()),
                None => bail!(format!(
                    "Node is missing required property {} for identity",
                    prop_name
                )),
            }
        }

        hasher.update(node.node_type.as_bytes());

        Ok(hex::encode(hasher.finalize()))
    }

    #[tracing::instrument(skip(self, strategy), err)]
    pub(crate) async fn attribute_dynamic_session(
        &self,
        node: NodeDescription,
        strategy: &Session,
    ) -> Result<NodeDescription, Error> {
        let mut attributed_node = node.clone();

        let primary_key = self
            .primary_session_key(&mut attributed_node, strategy)
            .await?;

        let created_time = strategy.create_time;
        let last_seen_time = strategy.last_seen_time;

        let unid = match (created_time != 0, last_seen_time != 0) {
            (true, _) => UnidSession {
                pseudo_key: primary_key,
                timestamp: created_time,
                is_creation: true,
            },
            (_, true) => UnidSession {
                pseudo_key: primary_key,
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

        attributed_node.node_key = session_id;

        Ok(attributed_node)
    }

    #[tracing::instrument(skip(self, node, strategy), err)]
    pub(crate) async fn attribute_static_mapping(
        &self,
        mut node: NodeDescription,
        strategy: &Static,
    ) -> Result<NodeDescription, Error> {
        let static_node_key = self.get_static_node_key(&node, strategy)?;
        node.set_key(static_node_key);

        Ok(node)
    }

    #[tracing::instrument(skip(self, node), err)]
    pub(crate) async fn attribute_dynamic_node(
        &self,
        node: &NodeDescription,
    ) -> Result<NodeDescription, Error> {
        let mut attributed_node = node.clone();
        let strategy = &node.id_strategy[0];

        match strategy.strategy {
            Strategy::Session(ref strategy) => {
                tracing::info!("Attributing dynamic node via session");
                attributed_node = self
                    .attribute_dynamic_session(attributed_node, strategy)
                    .await?;
            }
            Strategy::Static(ref strategy) => {
                tracing::info!("Attributing dynamic node via static mapping");
                attributed_node = self
                    .attribute_static_mapping(attributed_node, strategy)
                    .await?;
            }
        }

        Ok(attributed_node)
    }

    #[tracing::instrument(skip(self, unid_graph, _unid_id_map))]
    pub(crate) async fn attribute_dynamic_nodes(
        &self,
        unid_graph: GraphDescription,
        _unid_id_map: &mut HashMap<String, String>,
    ) -> Result<GraphDescription, GraphDescription> {
        let mut unid_id_map = HashMap::new();
        let mut dead_nodes: HashSet<&str> = HashSet::new();
        let mut output_graph = GraphDescription::new();
        output_graph.edges = unid_graph.edges;

        for node in unid_graph.nodes.values() {
            let span = tracing::trace_span!("dynamic attribution loop", node_key=?node.node_key);
            let _enter = span.enter();
            let new_node = match self.attribute_dynamic_node(node).await {
                Ok(node) => node,
                Err(e) => {
                    tracing::warn!(message="Failed to attribute dynamic node", error=?e);
                    dead_nodes.insert(node.node_key.as_ref());
                    continue;
                }
            };

            tracing::info!(message="Attributed NodeDescription", old_key=?node.node_key, new_key=?new_node.node_key);

            unid_id_map.insert(node.clone_node_key(), new_node.clone_node_key());
            output_graph.add_node(new_node);
        }

        if dead_nodes.is_empty() {
            tracing::info!("Attributed all dynamic nodes");
            Ok(output_graph)
        } else {
            tracing::warn!(message="Failed to attribute dynamic nodes", dead_nodes=?dead_nodes.len());
            Err(output_graph)
        }
    }
}
