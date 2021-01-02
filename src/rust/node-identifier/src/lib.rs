use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::io::{Cursor, Stdout};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use aws_lambda_events::event::s3::{
    S3Bucket, S3Entity, S3Event, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity,
};
use aws_lambda_events::event::sqs::SqsEvent;
use bytes::Bytes;
use chrono::Utc;
use failure::{bail, Error};
use lambda_runtime::error::HandlerError;
use lambda_runtime::Context;
use log::*;
use prost::Message;
use rusoto_core::{HttpClient, Region};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
use rusoto_s3::S3Client;
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use sha2::Digest;

use assetdb::{AssetIdDb, AssetIdentifier};
use dynamic_sessiondb::{DynamicMappingDb, DynamicNodeIdentifier};
use grapl_config::env_helpers::{s3_event_emitters_from_env, FromEnv};
use grapl_config::event_caches;
use grapl_graph_descriptions::file::FileState;
use grapl_graph_descriptions::graph_description::host::*;
use grapl_graph_descriptions::graph_description::node::WhichNode;
use grapl_graph_descriptions::graph_description::*;
use grapl_graph_descriptions::ip_connection::IpConnectionState;
use grapl_graph_descriptions::network_connection::NetworkConnectionState;
use grapl_graph_descriptions::node::NodeT;
use grapl_graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use grapl_graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use grapl_observe::metric_reporter::MetricReporter;
use sessiondb::SessionDb;
use sessions::UnidSession;
use sqs_executor::cache::{Cache, CacheResponse, Cacheable};
use sqs_executor::completion_event_serializer::CompletionEventSerializer;
use sqs_executor::errors::{CheckedError, Recoverable};
use sqs_executor::event_decoder::PayloadDecoder;
use sqs_executor::event_handler::{CompletedEvents, EventHandler};
use sqs_executor::event_retriever::S3PayloadRetriever;
use sqs_executor::redis_cache::RedisCache;
use sqs_executor::s3_event_emitter::{S3EventEmitter, S3ToSqsEventNotifier};
use sqs_executor::{make_ten, time_based_key_fn};

use grapl_service::decoder::ZstdProtoDecoder;
use grapl_service::serialization::SubgraphSerializer;
use sqs_executor::event_status::EventStatus;

macro_rules! wait_on {
    ($x:expr) => {{
        $x.await
    }};
}

pub mod assetdb;
pub mod dynamic_sessiondb;

pub mod sessiondb;
pub mod sessions;

#[derive(Clone)]
pub struct NodeIdentifier<D, CacheT>
where
    D: DynamoDb + Clone + Send + Sync + 'static,
    CacheT: Cache + Clone + Send + Sync + 'static,
{
    asset_mapping_db: AssetIdDb<D>,
    dynamic_identifier: DynamicNodeIdentifier<D>,
    asset_identifier: AssetIdentifier<D>,
    node_id_db: D,
    should_default: bool,
    cache: CacheT,
}

impl<D, CacheT> NodeIdentifier<D, CacheT>
where
    D: DynamoDb + Clone + Send + Sync + 'static,
    CacheT: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(
        asset_mapping_db: AssetIdDb<D>,
        dynamic_identifier: DynamicNodeIdentifier<D>,
        asset_identifier: AssetIdentifier<D>,
        node_id_db: D,
        should_default: bool,
        cache: CacheT,
    ) -> Self {
        Self {
            asset_mapping_db,
            dynamic_identifier,
            asset_identifier,
            node_id_db,
            should_default,
            cache,
        }
    }

    async fn attribute_node_key(&self, node: Node) -> Result<Node, Error> {
        let unid = into_unid_session(&node)?;

        match node.which_node {
            Some(WhichNode::ProcessNode(mut process_node)) => {
                info!("Attributing ProcessNode: {}", process_node.process_id);
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify ProcessNode"),
                };
                let session_db = SessionDb::new(
                    self.node_id_db.clone(),
                    grapl_config::process_history_table_name(),
                );
                let node_key = session_db
                    .handle_unid_session(unid, self.should_default)
                    .await?;

                info!("Mapped Process {:?} to {}", process_node, &node_key,);
                process_node.set_node_key(node_key);
                Ok(process_node.into())
            }
            Some(WhichNode::FileNode(mut file_node)) => {
                info!("Attributing FileNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify FileNode"),
                };
                let session_db = SessionDb::new(
                    self.node_id_db.clone(),
                    grapl_config::file_history_table_name(),
                );
                let node_key = session_db
                    .handle_unid_session(unid, self.should_default)
                    .await?;

                file_node.set_node_key(node_key);
                Ok(file_node.into())
            }
            Some(WhichNode::ProcessInboundConnectionNode(mut inbound_node)) => {
                info!("Attributing ProcessInboundConnectionNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify ProcessInboundConnectionNode"),
                };
                let session_db = SessionDb::new(
                    self.node_id_db.clone(),
                    grapl_config::inbound_connection_history_table_name(),
                );
                let node_key = session_db
                    .handle_unid_session(unid, self.should_default)
                    .await?;

                inbound_node.set_node_key(node_key);
                Ok(inbound_node.into())
            }
            Some(WhichNode::ProcessOutboundConnectionNode(mut outbound_node)) => {
                info!("Attributing ProcessOutboundConnectionNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify ProcessOutboundConnectionNode"),
                };
                let session_db = SessionDb::new(
                    self.node_id_db.clone(),
                    grapl_config::outbound_connection_history_table_name(),
                );
                let node_key = session_db
                    .handle_unid_session(unid, self.should_default)
                    .await?;

                outbound_node.set_node_key(node_key);
                Ok(outbound_node.into())
            }
            Some(WhichNode::AssetNode(mut asset_node)) => {
                info!("Attributing AssetNode");
                let asset_id = match asset_node.clone_asset_id() {
                    Some(asset_id) => asset_id,
                    None => bail!("AssetNode must have asset_id"),
                };

                // AssetNodes have a node_key equal to their asset_id
                asset_node.set_node_key(asset_id);
                Ok(asset_node.into())
            }
            // IpAddress nodes are identified at construction
            Some(WhichNode::IpAddressNode(mut ip_node)) => {
                ip_node.set_node_key(ip_node.ip_address.clone());
                info!("Attributing IpAddressNode");
                Ok(ip_node.into())
            }
            // The identity of an IpPortNode is the hash of its ip, port, and protocol
            Some(WhichNode::IpPortNode(mut ip_port)) => {
                info!("Attributing IpPortNode");
                let port = &ip_port.port;
                let protocol = &ip_port.protocol;

                let mut node_key_hasher = sha2::Sha256::default();
                node_key_hasher.input(port.to_string().as_bytes());
                node_key_hasher.input(protocol.as_bytes());

                let node_key = hex::encode(node_key_hasher.result());

                ip_port.set_node_key(node_key);

                Ok(ip_port.into())
            }
            Some(WhichNode::NetworkConnectionNode(mut network_connection_node)) => {
                info!("Attributing NetworkConnectionNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify NetworkConnectionNode"),
                };
                let session_db = SessionDb::new(
                    self.node_id_db.clone(),
                    grapl_config::network_connection_history_table_name(),
                );
                let node_key = session_db
                    .handle_unid_session(unid, self.should_default)
                    .await?;

                network_connection_node.set_node_key(node_key);
                Ok(network_connection_node.into())
            }
            Some(WhichNode::IpConnectionNode(mut ip_connection_node)) => {
                info!("Attributing IpConnectionNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify IpConnectionNode"),
                };
                let session_db = SessionDb::new(
                    self.node_id_db.clone(),
                    grapl_config::ip_connection_history_table_name(),
                );
                let node_key = session_db
                    .handle_unid_session(unid, self.should_default)
                    .await?;

                ip_connection_node.set_node_key(node_key);
                Ok(ip_connection_node.into())
            }
            Some(WhichNode::DynamicNode(ref dynamic_node)) => {
                info!("Attributing DynamicNode");
                let new_node = self
                    .dynamic_identifier
                    .attribute_dynamic_node(&dynamic_node)
                    .await?;
                Ok(new_node.into())
            }
            None => bail!("Unknown Node Variant"),
        }
    }
}

fn into_unid_session(node: &Node) -> Result<Option<UnidSession>, Error> {
    match &node.which_node {
        Some(WhichNode::ProcessNode(node)) => {
            let (is_creation, timestamp) = match (
                node.created_timestamp != 0,
                node.last_seen_timestamp != 0,
                node.terminated_timestamp != 0,
            ) {
                (true, _, _) => (true, node.created_timestamp),
                (_, _, true) => (false, node.terminated_timestamp),
                (_, true, _) => (false, node.last_seen_timestamp),
                _ => bail!("At least one timestamp must be set"),
            };

            Ok(Some(UnidSession {
                pseudo_key: format!(
                    "{}{}",
                    node.get_asset_id().expect("ProcessNode must have asset_id"),
                    node.process_id
                ),
                timestamp,
                is_creation,
            }))
        }
        Some(WhichNode::FileNode(node)) => {
            let (is_creation, timestamp) = match FileState::try_from(node.state)? {
                FileState::Created => (true, node.created_timestamp),
                _ => (false, node.last_seen_timestamp),
            };
            // TODO: Hash the path
            let key = &node.file_path;

            Ok(Some(UnidSession {
                pseudo_key: format!(
                    "{}{}",
                    node.get_asset_id().expect("FileNode must have asset_id"),
                    key
                ),
                timestamp,
                is_creation,
            }))
        }
        Some(WhichNode::ProcessOutboundConnectionNode(node)) => {
            let (is_creation, timestamp) =
                match ProcessOutboundConnectionState::try_from(node.state)? {
                    ProcessOutboundConnectionState::Connected => (true, node.created_timestamp),
                    _ => (false, node.last_seen_timestamp),
                };

            Ok(Some(UnidSession {
                pseudo_key: format!(
                    "{}{}outbound",
                    node.get_asset_id()
                        .expect("ProcessOutboundConnectionNode must have asset_id"),
                    node.port
                ),
                timestamp,
                is_creation,
            }))
        }
        Some(WhichNode::ProcessInboundConnectionNode(node)) => {
            let (is_creation, timestamp) =
                match ProcessInboundConnectionState::try_from(node.state)? {
                    ProcessInboundConnectionState::Bound => (true, node.created_timestamp),
                    _ => (false, node.last_seen_timestamp),
                };

            Ok(Some(UnidSession {
                pseudo_key: format!(
                    "{}{}inbound",
                    node.get_asset_id().expect("Missing asset id"),
                    node.port
                ),
                timestamp,
                is_creation,
            }))
        }

        Some(WhichNode::NetworkConnectionNode(node)) => {
            let (is_creation, timestamp) = match NetworkConnectionState::try_from(node.state)? {
                NetworkConnectionState::Created => (true, node.created_timestamp),
                _ => (false, node.last_seen_timestamp),
            };

            let pseudo_key = format!(
                "{}{}{}{}{}network_connection",
                node.src_port,
                node.src_ip_address,
                node.dst_port,
                node.dst_ip_address,
                node.protocol,
            );
            Ok(Some(UnidSession {
                pseudo_key,
                timestamp,
                is_creation,
            }))
        }

        Some(WhichNode::IpConnectionNode(node)) => {
            let (is_creation, timestamp) = match IpConnectionState::try_from(node.state)? {
                IpConnectionState::Created => (true, node.created_timestamp),
                _ => (false, node.last_seen_timestamp),
            };

            let pseudo_key = format!(
                "{}{}{}ip_network_connection",
                node.src_ip_address, node.dst_ip_address, node.protocol,
            );
            Ok(Some(UnidSession {
                pseudo_key,
                timestamp,
                is_creation,
            }))
        }
        // IpAddressNode is not a session
        Some(WhichNode::IpAddressNode(_node)) => Ok(None),

        // AssetNode is not a session
        Some(WhichNode::AssetNode(_node)) => Ok(None),

        // IpPortNode is not a session
        Some(WhichNode::IpPortNode(_node)) => Ok(None),

        // DynamicNode's are identified separatealy from others
        Some(WhichNode::DynamicNode(_node)) => Ok(None),
        None => bail!("Failed to handle variant of node. Dropping it."),
    }
}

fn remove_dead_nodes(graph: &mut Graph, dead_nodes: &HashSet<impl Deref<Target = str>>) {
    for dead_node in dead_nodes {
        graph.nodes.remove(dead_node.deref());
        graph.edges.remove(dead_node.deref());
    }
}

fn remove_dead_edges(graph: &mut Graph) {
    let edges = &mut graph.edges;
    let nodes = &graph.nodes;
    for (_node_key, edge_list) in edges.iter_mut() {
        let live_edges: Vec<_> = edge_list
            .edges
            .clone()
            .into_iter()
            .filter(|edge| nodes.contains_key(&edge.to) && nodes.contains_key(&edge.from))
            .collect();

        *edge_list = EdgeList { edges: live_edges };
    }
}

fn remap_edges(graph: &mut Graph, unid_id_map: &HashMap<String, String>) {
    for (_node_key, edge_list) in graph.edges.iter_mut() {
        for edge in edge_list.edges.iter_mut() {
            let from = match unid_id_map.get(&edge.from) {
                Some(from) => from,
                None => {
                    warn!(
                        "Failed to lookup from node in unid_id_map {}",
                        &edge.edge_name
                    );
                    continue;
                }
            };

            let to = match unid_id_map.get(&edge.to) {
                Some(to) => to,
                None => {
                    warn!(
                        "Failed to lookup to node in unid_id_map {}",
                        &edge.edge_name
                    );
                    continue;
                }
            };

            *edge = Edge {
                from: from.to_owned(),
                to: to.to_owned(),
                edge_name: edge.edge_name.clone(),
            };
        }
    }
}

fn remap_nodes(graph: &mut Graph, unid_id_map: &HashMap<String, String>) {
    let mut nodes = HashMap::with_capacity(graph.nodes.len());

    for (_node_key, node) in graph.nodes.iter_mut() {
        // DynamicNodes are identified in-place
        if let Some(_n) = node.as_dynamic_node() {
            let old_node = nodes.insert(node.clone_node_key(), node.clone());
            if let Some(ref old_node) = old_node {
                NodeT::merge(
                    nodes
                        .get_mut(node.get_node_key())
                        .expect("node key not in map"),
                    old_node,
                );
            }
        } else if let Some(new_key) = unid_id_map.get(node.get_node_key()) {
            node.set_node_key(new_key.to_owned());

            // We may have actually had nodes with different unid node_keys that map to the
            // same node_key. Therefor we must merge any nodes when there is a collision.
            let old_node = nodes.insert(new_key.to_owned(), node.clone());
            if let Some(ref old_node) = old_node {
                NodeT::merge(
                    nodes.get_mut(new_key).expect("New key not in map"),
                    old_node,
                );
            }
        }
    }
    graph.nodes = nodes;
}

async fn create_asset_id_mappings(
    assetid_db: &AssetIdDb<impl DynamoDb>,
    unid_graph: &Graph,
) -> Result<(), Error> {
    for node in unid_graph.nodes.values() {
        let ids = match &node.which_node {
            Some(WhichNode::ProcessNode(ref node)) => {
                (&node.asset_id, &node.hostname, node.created_timestamp)
            }
            Some(WhichNode::FileNode(ref node)) => {
                (&node.asset_id, &node.hostname, node.created_timestamp)
            }
            Some(WhichNode::ProcessOutboundConnectionNode(ref node)) => {
                (&node.asset_id, &node.hostname, node.created_timestamp)
            }
            Some(WhichNode::ProcessInboundConnectionNode(ref node)) => {
                (&node.asset_id, &node.hostname, node.created_timestamp)
            }
            Some(WhichNode::AssetNode(ref node)) => {
                (&node.asset_id, &node.hostname, node.first_seen_timestamp)
            }
            Some(WhichNode::NetworkConnectionNode(ref _node)) => {
                continue;
            }
            Some(WhichNode::IpConnectionNode(ref _node)) => {
                continue;
            }
            Some(WhichNode::IpAddressNode(ref _node)) => {
                continue;
            }
            Some(WhichNode::IpPortNode(ref _node)) => {
                continue;
            }
            Some(WhichNode::DynamicNode(ref _node)) => {
                continue;
            }
            None => bail!("Failed to handle node variant"),
        };

        match ids {
            (Some(asset_id), Some(hostname), timestamp) => {
                info!("Creating asset id {} mapping for: {}", asset_id, hostname);
                assetid_db
                    .create_mapping(
                        &HostId::AssetId(asset_id.clone()),
                        hostname.clone(),
                        timestamp,
                    )
                    .await?;
            }
            _ => continue,
        };
    }

    Ok(())
}

// Takes a Graph, attributes all nodes with an asset id
// When atribution fails, attribution continues, but the Graph returned will contain
// only the nodes that were successful
// Edges will also be fixed up
async fn attribute_asset_ids(
    asset_identifier: &AssetIdentifier<impl DynamoDb>,
    unid_graph: Graph,
) -> Result<Graph, (Error, Graph)> {
    info!("Attributing asset ids");
    let mut dead_nodes = HashSet::new();
    let mut output_graph = Graph::new(unid_graph.timestamp);
    output_graph.edges = unid_graph.edges;

    let _node_asset_ids: HashMap<String, String> = HashMap::new();
    let mut err = None;

    for node in unid_graph.nodes.values() {
        match &node.which_node {
            Some(WhichNode::IpAddressNode(n)) => {
                output_graph.add_node(n.clone());
                continue;
            }
            Some(WhichNode::DynamicNode(n)) => {
                if !n.requires_asset_identification() {
                    output_graph.add_node(n.clone());
                    continue;
                }
            }
            Some(WhichNode::NetworkConnectionNode(n)) => {
                output_graph.add_node(n.clone());
                continue;
            }
            Some(WhichNode::IpPortNode(n)) => {
                output_graph.add_node(n.clone());
                continue;
            }
            Some(WhichNode::IpConnectionNode(n)) => {
                output_graph.add_node(n.clone());
                continue;
            }
            _ => (),
        }

        let asset_id = asset_identifier.attribute_asset_id(&node).await;

        let asset_id = match asset_id {
            Ok(asset_id) => asset_id,
            Err(e) => {
                warn!("Failed to attribute to asset id: {:?} {}", node, e);
                err = Some(e);
                dead_nodes.insert(node.clone_node_key());
                continue;
            }
        };

        let mut node = node.to_owned();
        node.set_asset_id(asset_id);
        output_graph.add_node(node);
    }

    // There shouldn't be any dead nodes in our output_graph anyways
    remove_dead_edges(&mut output_graph);

    if dead_nodes.is_empty() {
        info!("Attributed all asset ids");
        Ok(output_graph)
    } else {
        warn!("Attributed asset ids");
        Err((err.unwrap(), output_graph))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NodeIdentifierError {
    #[error("Unexpected error")]
    Unexpected,
}

impl CheckedError for NodeIdentifierError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

#[async_trait]
impl<D, CacheT> EventHandler for NodeIdentifier<D, CacheT>
where
    D: DynamoDb + Clone + Send + Sync + 'static,
    CacheT: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = Graph;
    type Error = NodeIdentifierError;

    async fn handle_event(
        &mut self,
        subgraphs: GeneratedSubgraphs,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        warn!("node-identifier.handle_event");

        let mut attribution_failure = None;

        info!("Handling raw event");

        if subgraphs.subgraphs.is_empty() {
            warn!("Received empty unid subgraph");
            return Ok(Graph::new(0));
        }

        // Merge all of the subgraphs into one subgraph to avoid
        // redundant work
        let unid_subgraph =
            subgraphs
                .subgraphs
                .into_iter()
                .fold(Graph::new(0), |mut total_graph, subgraph| {
                    info!(
                        "Merging subgraph with: {} nodes {} edges",
                        subgraph.nodes.len(),
                        subgraph.edges.len()
                    );
                    total_graph.merge(&subgraph);
                    total_graph
                });

        if unid_subgraph.is_empty() {
            warn!("Received empty subgraph");
            return Ok(Graph::new(0));
        }

        info!(
            "unid_subgraph: {} nodes {} edges",
            unid_subgraph.nodes.len(),
            unid_subgraph.edges.len(),
        );

        // Create any implicit asset id mappings
        if let Err(e) = create_asset_id_mappings(&self.asset_mapping_db, &unid_subgraph).await {
            error!("Asset mapping creation failed with {}", e);
            return Err(Err(NodeIdentifierError::Unexpected));
        }

        // Map all host_ids into asset_ids. This has to happen before node key
        // identification.
        // If there is a failure, we'll mark this execute as failed, but continue
        // with whatever subgraph has succeeded

        let output_subgraph = match attribute_asset_ids(&self.asset_identifier, unid_subgraph).await
        {
            Ok(unid_subgraph) => unid_subgraph,
            Err((e, unid_subgraph)) => {
                attribution_failure = Some(e);
                unid_subgraph
            }
        };

        let mut dead_node_ids = HashSet::new();
        let mut unid_id_map = HashMap::new();

        // new method
        let mut identified_graph = Graph::new(output_subgraph.timestamp);
        for (old_node_key, old_node) in output_subgraph.nodes.iter() {
            let node = old_node.clone();

            match self.cache.get(old_node_key.clone()).await {
                Ok(CacheResponse::Hit) => {
                    info!("Got cache hit for old_node_key, skipping node.");
                    continue;
                }
                Err(e) => warn!("Failed to retrieve from cache: {:?}", e),
                _ => (),
            };

            let node = match self.attribute_node_key(node.clone()).await {
                Ok(node) => node,
                Err(e) => {
                    warn!("Failed to attribute node_key with: {}", e);
                    dead_node_ids.insert(node.clone_node_key());
                    completed.add_identity(node.clone_node_key(), EventStatus::Failure);
                    attribution_failure = Some(e);
                    continue;
                }
            };
            unid_id_map.insert(old_node_key.to_owned(), node.clone_node_key());
            identified_graph.add_node(node);
        }

        info!(
            "PRE: identified_graph.edges.len() {}",
            identified_graph.edges.len()
        );

        for (old_key, edge_list) in output_subgraph.edges.iter() {
            if dead_node_ids.contains(old_key) {
                continue;
            };

            for edge in &edge_list.edges {
                let from_key = unid_id_map.get(&edge.from);
                let to_key = unid_id_map.get(&edge.to);

                let (from_key, to_key) = match (from_key, to_key) {
                    (Some(from_key), Some(to_key)) => (from_key, to_key),
                    _ => continue,
                };

                identified_graph.add_edge(
                    edge.edge_name.to_owned(),
                    from_key.to_owned(),
                    to_key.to_owned(),
                );
            }
        }

        info!(
            "POST: identified_graph.edges.len() {}",
            identified_graph.edges.len()
        );

        // Remove dead nodes and edges from output_graph
        let dead_node_ids: HashSet<&str> = dead_node_ids.iter().map(String::as_str).collect();

        if identified_graph.is_empty() {
            return Err(Err(NodeIdentifierError::Unexpected));
        }

        let identities: Vec<_> = unid_id_map.keys().collect();

        identities
            .iter()
            .for_each(|identity| completed.add_identity(identity.clone(), EventStatus::Success));

        if !dead_node_ids.is_empty() || attribution_failure.is_some() {
            info!("Partial Success, identified {} nodes", identities.len());
            Err(Ok(
                (identified_graph, NodeIdentifierError::Unexpected), // todo: Use a real error here
            ))
        } else {
            info!("Identified all nodes");
            Ok(identified_graph)
        }
    }
}

pub async fn handler(should_default: bool) -> Result<(), HandlerError> {
    let env = grapl_config::init_grapl_env!();
    let source_queue_url = grapl_config::source_queue_url();
    debug!("Queue Url: {}", source_queue_url);

    let sqs_client = SqsClient::from_env();
    let s3_client = S3Client::from_env();

    let destination_bucket = grapl_config::dest_bucket();
    let cache = &mut event_caches(&env).await;

    let serializer = &mut make_ten(async { SubgraphSerializer::default() }).await;

    let s3_emitter =
        &mut s3_event_emitters_from_env(&env, time_based_key_fn, S3ToSqsEventNotifier::from_env())
            .await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| grapl_config::env_helpers::init_s3_client(&region_str),
            ZstdProtoDecoder::default(),
            MetricReporter::new(&env.service_name),
        )
    })
    .await;
    info!("Output events to: {}", destination_bucket);

    let asset_id_db = AssetIdDb::new(DynamoDbClient::from_env());

    let dynamo = DynamoDbClient::from_env();
    let dyn_session_db = SessionDb::new(dynamo.clone(), grapl_config::dynamic_session_table_name());
    let dyn_mapping_db = DynamicMappingDb::new(DynamoDbClient::from_env());
    let asset_identifier = AssetIdentifier::new(asset_id_db);

    let dyn_node_identifier = DynamicNodeIdentifier::new(
        asset_identifier,
        dyn_session_db,
        dyn_mapping_db,
        should_default,
    );

    let asset_id_db = AssetIdDb::new(DynamoDbClient::from_env());

    let asset_identifier = AssetIdentifier::new(asset_id_db);

    let asset_id_db = AssetIdDb::new(DynamoDbClient::from_env());
    let node_identifier = &mut make_ten(async {
        NodeIdentifier::new(
            asset_id_db,
            dyn_node_identifier,
            asset_identifier,
            dynamo.clone(),
            should_default,
            cache[0].to_owned(),
        )
    })
    .await;

    info!("Starting process_loop");
    sqs_executor::process_loop(
        grapl_config::source_queue_url(),
        std::env::var("DEAD_LETTER_QUEUE_URL").expect("DEAD_LETTER_QUEUE_URL"),
        cache,
        sqs_client.clone(),
        node_identifier,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
    .await;

    info!("Exiting");
    println!("Exiting");
    Ok(())
}

#[derive(Clone, Default)]
pub struct HashCache {
    cache: Arc<Mutex<std::collections::HashSet<Vec<u8>>>>,
}

#[derive(thiserror::Error, Debug)]
pub enum HashCacheError {
    #[error("Unreachable error")]
    Unreachable,
}

impl CheckedError for HashCacheError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Persistent
    }
}

#[async_trait]
impl Cache for HashCache {
    type CacheErrorT = HashCacheError;

    async fn get<CA: Cacheable + Send + Sync + 'static>(
        &mut self,
        cacheable: CA,
    ) -> Result<CacheResponse, HashCacheError> {
        let self_cache = self.cache.lock().unwrap();

        let id = cacheable.identity();
        if self_cache.contains(&id) {
            Ok(CacheResponse::Hit)
        } else {
            Ok(CacheResponse::Miss)
        }
    }
    async fn store(&mut self, identity: Vec<u8>) -> Result<(), HashCacheError> {
        let mut self_cache = self.cache.lock().unwrap();
        self_cache.insert(identity);
        Ok(())
    }
}
