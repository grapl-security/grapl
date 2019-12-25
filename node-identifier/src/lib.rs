#![allow(dead_code)]

extern crate hex;
extern crate aws_lambda_events;
extern crate base58;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate graph_descriptions;

extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;

extern crate prost;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
extern crate rusoto_core;
extern crate rusoto_dynamodb;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_dynamodb;
extern crate serde_json;
extern crate sha2;
extern crate simple_logger;
extern crate sqs_lambda;
extern crate stopwatch;
extern crate tokio;
extern crate uuid;
extern crate zstd;
#[macro_use]
extern crate hmap;



use std::collections::HashMap;
use std::collections::HashSet;
use std::env;

use std::io::Cursor;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;


use aws_lambda_events::event::sqs::SqsEvent;

use failure::Error;


use graph_descriptions::graph_description::*;
use graph_descriptions::graph_description::host::*;
use lambda::Context;
use lambda::error::HandlerError;

use prost::Message;
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;
use sha2::Digest;


use assetdb::{AssetIdDb, AssetIdentifier};
use sessiondb::SessionDb;
use sessions::UnidSession;
use dynamic_sessiondb::{DynamicNodeIdentifier, DynamicMappingDb};
use graph_descriptions::graph_description::node::WhichNode;
use graph_descriptions::process::ProcessState;
use graph_descriptions::file::FileState;
use graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use graph_descriptions::network_connection::NetworkConnectionState;

use std::convert::TryFrom;
use graph_descriptions::ip_connection::IpConnectionState;

use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::event_emitter::S3EventEmitter;
use sqs_lambda::event_handler::{EventHandler, OutputEvent, Completion};

use sqs_lambda::event_processor::{EventProcessor, EventProcessorActor};
use sqs_lambda::event_retriever::S3PayloadRetriever;

use sqs_lambda::sqs_completion_handler::{CompletionPolicy, SqsCompletionHandler, SqsCompletionHandlerActor};
use sqs_lambda::sqs_consumer::{ConsumePolicy, SqsConsumer, SqsConsumerActor};

use async_trait::async_trait;

use crate::graph_descriptions::node::NodeT;


use sqs_lambda::redis_cache::RedisCache;



macro_rules! log_time {
    ($msg:expr, $x:expr) => {{
        let mut sw = Stopwatch::start_new();
        #[allow(path_statements)]
        let result = $x;
        sw.stop();
        info!("{} {} milliseconds", $msg, sw.elapsed_ms());
        result
    }};
}

macro_rules! wait_on {
    ($x:expr) => {{
        //            let rt = tokio::runtime::current_thread::Runtime::new()?;
        futures::compat::Future01CompatExt::compat(
            $x.with_timeout(Duration::from_secs(2))
        )
        .await
    }};
}

pub mod assetdb;
pub mod dynamic_sessiondb;
pub mod retry_cache;
pub mod sessiondb;
pub mod sessions;

#[derive(Clone)]
struct NodeIdentifier<D>
    where
        D: DynamoDb + Clone + Send + Sync + 'static,
{
    asset_mapping_db: AssetIdDb<D>,
    dynamic_identifier: DynamicNodeIdentifier<D>,
    asset_identifier: AssetIdentifier<D>,
    node_id_db: D,
    should_default: bool,
}

impl<D> NodeIdentifier<D>
    where
        D: DynamoDb + Clone + Send + Sync + 'static,
        
{
    pub fn new(
        asset_mapping_db: AssetIdDb<D>,
        dynamic_identifier: DynamicNodeIdentifier<D>,
        asset_identifier: AssetIdentifier<D>,
        node_id_db: D,
        should_default: bool,
    ) -> Self {
        Self {
            asset_mapping_db,
            dynamic_identifier,
            asset_identifier,
            node_id_db,
            should_default,
        }
    }

    async fn attribute_node_key(&self, node: Node) -> Result<Node, Error> {
        let unid = into_unid_session(&node)?;

        match node.which_node {
            Some(WhichNode::ProcessNode(mut process_node)) => {
                info!("Attributing ProcessNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify ProcessNode")
                };
                let session_db = SessionDb::new(self.node_id_db.clone(), "process_history_table");
                let node_key = session_db.handle_unid_session(unid, self.should_default).await?;

                process_node.set_node_key(node_key);
                Ok(process_node.into())
            }
            Some(WhichNode::FileNode(mut file_node)) => {
                info!("Attributing FileNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify FileNode")
                };
                let session_db = SessionDb::new(self.node_id_db.clone(), "file_history_table");
                let node_key = session_db.handle_unid_session(unid, self.should_default).await?;

                file_node.set_node_key(node_key);
                Ok(file_node.into())
            }
            Some(WhichNode::ProcessInboundConnectionNode(mut inbound_node)) => {
                info!("Attributing ProcessInboundConnectionNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify ProcessInboundConnectionNode")
                };
                let session_db = SessionDb::new(self.node_id_db.clone(), "outbound_connection_history_table");
                let node_key = session_db.handle_unid_session(unid, self.should_default).await?;

                inbound_node.set_node_key(node_key);
                Ok(inbound_node.into())
            }
            Some(WhichNode::ProcessOutboundConnectionNode(mut outbound_node)) => {
                info!("Attributing ProcessOutboundConnectionNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify ProcessOutboundConnectionNode")
                };
                let session_db = SessionDb::new(self.node_id_db.clone(), "outbound_connection_history_table");
                let node_key = session_db.handle_unid_session(unid, self.should_default).await?;

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
            Some(WhichNode::IpAddressNode(_)) => {
                info!("Attributing IpAddressNode");
                Ok(node)
            }
            // The identity of an IpPortNode is the hash of its ip, port, and protocol
            Some(WhichNode::IpPortNode(mut ip_port)) => {
                info!("Attributing IpPortNode");
                let ip_address = &ip_port.ip_address;
                let port = &ip_port.port;
                let protocol = &ip_port.protocol;

                let mut node_key_hasher = sha2::Sha256::default();
                node_key_hasher.input(ip_address.as_bytes());
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
                    None => bail!("Could not identify NetworkConnectionNode")
                };
                let session_db = SessionDb::new(self.node_id_db.clone(), "network_connection_history_table");
                let node_key = session_db.handle_unid_session(unid, self.should_default).await?;

                network_connection_node.set_node_key(node_key);
                Ok(network_connection_node.into())
            }
            Some(WhichNode::IpConnectionNode(mut ip_connection_node)) => {
                info!("Attributing IpConnectionNode");
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify IpConnectionNode")
                };
                let session_db = SessionDb::new(self.node_id_db.clone(), "ip_connection_history_table");
                let node_key = session_db.handle_unid_session(unid, self.should_default).await?;

                ip_connection_node.set_node_key(node_key);
                Ok(ip_connection_node.into())
            }
            Some(WhichNode::DynamicNode(ref dynamic_node)) => {
                info!("Attributing DynamicNode");
                let new_node = self.dynamic_identifier.attribute_dynamic_node(&dynamic_node).await?;
                Ok(new_node.into())
            }
            None => bail!("Unknown Node Variant")
        }
    }
}

fn into_unid_session(node: &Node) -> Result<Option<UnidSession>, Error> {
    match &node.which_node {
        Some(WhichNode::ProcessNode(node)) => {
            let (is_creation, timestamp) = match ProcessState::try_from(node.state)? {
                ProcessState::Created => (true, node.created_timestamp),
                _ => (false, node.last_seen_timestamp),
            };

            Ok(
                Some(
                    UnidSession {
                        pseudo_key: format!(
                            "{}{}",
                            node.get_asset_id().expect("ProcessNode must have asset_id"),
                            node.process_id
                        ),
                        timestamp,
                        is_creation,
                    }
                )
            )
        }
        Some(WhichNode::FileNode(node)) => {
            let (is_creation, timestamp) = match FileState::try_from(node.state)? {
                FileState::Created => (true, node.created_timestamp),
                _ => (false, node.last_seen_timestamp),
            };
            // TODO: Hash the path
            let key = &node.file_path;

            Ok(
                Some(
                    UnidSession {
                        pseudo_key: format!("{}{}", node.get_asset_id().expect("FileNode must have asset_id"), key),
                        timestamp,
                        is_creation,
                    }
                )
            )
        }
        Some(WhichNode::ProcessOutboundConnectionNode(node)) => {
            let (is_creation, timestamp) = match ProcessOutboundConnectionState::try_from(node.state)? {
                ProcessOutboundConnectionState::Connected => (true, node.created_timestamp),
                _ => (false, node.last_seen_timestamp),
            };

            Ok(
                Some(
                    UnidSession {
                        pseudo_key: format!(
                            "{}{}outbound",
                            node.get_asset_id().expect("ProcessOutboundConnectionNode must have asset_id"),
                            node.port
                        ),
                        timestamp,
                        is_creation,
                    }
                )
            )
        }
        Some(WhichNode::ProcessInboundConnectionNode(node)) => {
            let (is_creation, timestamp) = match ProcessInboundConnectionState::try_from(node.state)? {
                ProcessInboundConnectionState::Bound => (true, node.created_timestamp),
                _ => (false, node.last_seen_timestamp),
            };

            Ok(
                Some(
                    UnidSession {
                        pseudo_key: format!(
                            "{}{}inbound",
                            node.get_asset_id().expect("Missing asset id"),
                            node.port
                        ),
                        timestamp,
                        is_creation,
                    }
                )
            )
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
            Ok(
                Some(
                    UnidSession {
                        pseudo_key,
                        timestamp,
                        is_creation,
                    }
                )
            )
        },

        Some(WhichNode::IpConnectionNode(node)) => {
            let (is_creation, timestamp) = match IpConnectionState::try_from(node.state)? {
                IpConnectionState::Created => (true, node.created_timestamp),
                _ => (false, node.last_seen_timestamp),
            };

            let pseudo_key = format!(
                "{}{}{}ip_network_connection",
                node.src_ip_address,
                node.dst_ip_address,
                node.protocol,
            );
            Ok(
                Some(
                    UnidSession {
                        pseudo_key,
                        timestamp,
                        is_creation,
                    }
                )
            )
        },
        // IpAddressNode is not a session
        Some(WhichNode::IpAddressNode(_node)) => Ok(None),

        // AssetNode is not a session
        Some(WhichNode::AssetNode(_node)) => Ok(None),

        // IpPortNode is not a session
        Some(WhichNode::IpPortNode(_node)) => Ok(None),

        // DynamicNode's are identified separatealy from others
        Some(WhichNode::DynamicNode(_node)) => {
            Ok(None)
        }
        None => bail!("Failed to handle variant of node. Dropping it.")
    }
}

fn remove_dead_nodes(graph: &mut Graph, dead_nodes: &HashSet<impl Deref<Target=str>>) {
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
    for (node_key, edge_list) in graph.edges.iter_mut() {
        for edge in edge_list.edges.iter_mut() {
            let from = match unid_id_map.get(&edge.from) {
                Some(from) => from,
                None => {
                    println!("Failed to lookup from node in unid_id_map {}", &edge.edge_name);
                    continue;
                }
            };

            let to = match unid_id_map.get(&edge.to) {
                Some(to) => to,
                None => {
                    println!("Failed to lookup to node in unid_id_map {}", &edge.edge_name);
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

    for (node_key, node) in graph.nodes.iter_mut() {
        // DynamicNodes are identified in-place
        if let Some(n) = node.as_dynamic_node() {
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
                    nodes
                        .get_mut(new_key)
                        .expect("New key not in map"),
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
            None => bail!("Failed to handle node variant")
        };

        match ids {
            (Some(asset_id), Some(hostname), timestamp) => {
                info!("Creating asset id {} mapping for: {}", asset_id, hostname);
                assetid_db.create_mapping(
                    &HostId::AssetId(asset_id.clone()),
                    hostname.clone(),
                    timestamp,
                ).await?;
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
) -> Result<Graph, Graph> {
    info!("Attributing asset ids");
    let mut dead_nodes = HashSet::new();
    let mut output_graph = Graph::new(unid_graph.timestamp);
    output_graph.edges = unid_graph.edges;

    let node_asset_ids: HashMap<String, String> = HashMap::new();

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
            _ => ()
        }

        let asset_id = asset_identifier.attribute_asset_id(
            &node,
        ).await;

        let asset_id = match asset_id {
            Ok(asset_id) => asset_id,
            Err(e) => {
                warn!("Failed to attribute to asset id: {:?} {}", node, e);
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
        Err(output_graph)
    }
}

#[async_trait]
impl<D> EventHandler for NodeIdentifier<D>
    where D: DynamoDb + Clone + Send + Sync + 'static,
{

    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = GeneratedSubgraphs;
    type Error = Arc<failure::Error>;

    async fn handle_event(&mut self, subgraphs: GeneratedSubgraphs) -> OutputEvent<Self::OutputEvent, Self::Error> {

        let region = {
            let region_str = env::var("AWS_REGION").expect("AWS_REGION");
            match Region::from_str(&region_str) {
                Ok(region) => region,
                Err(e) => return OutputEvent::new(Completion::Error(Arc::new(e.into())))
            }
        };

        let mut attribution_failure = false;

        info!("Handling raw event");

        if subgraphs.subgraphs.is_empty() {
            warn!("Received empty unid subgraph");
            return OutputEvent::new(Completion::Total(GeneratedSubgraphs::new(vec![])));
        }

        let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));
        let dynamo = DynamoDbClient::new(region.clone());
        let dyn_session_db = SessionDb::new(
            dynamo.clone(),
            "dynamic_session_table",
        );
        let dyn_mapping_db = DynamicMappingDb::new(DynamoDbClient::new(region.clone()));

        let retry_cache =
            retry_cache::RetrySessionCache::new("node_id_retry_table", DynamoDbClient::new(region));

        // Merge all of the subgraphs into one subgraph to avoid
        // redundant work
        let unid_subgraph = subgraphs.subgraphs.into_iter().fold(
            Graph::new(0),
            |mut total_graph, subgraph| {
                info!("Merging subgraph with: {} nodes {} edges", subgraph.nodes.len(), subgraph.edges.len());
                total_graph.merge(&subgraph);
                total_graph
            },
        );

        if unid_subgraph.is_empty() {
            warn!("Received empty subgraph");
            return OutputEvent::new(Completion::Total(GeneratedSubgraphs::new(vec![])));
        }


        info!(
            "unid_subgraph: {} nodes {} edges",
            unid_subgraph.nodes.len(),
            unid_subgraph.edges.len(),
        );

        // Create any implicit asset id mappings
        if let Err(e) = create_asset_id_mappings(&asset_id_db, &unid_subgraph).await {
            error!("Asset mapping creation failed with {}", e);
            return OutputEvent::new(Completion::Error(Arc::new(e.into())))
        }

        // Map all host_ids into asset_ids. This has to happen before node key
        // identification.
        // If there is a failure, we'll mark this execute as failed, but continue
        // with whatever subgraph has succeeded

        let output_subgraph = match attribute_asset_ids(&self.asset_identifier, unid_subgraph).await {
            Ok(unid_subgraph) => unid_subgraph,
            Err(unid_subgraph) => {
                attribution_failure = true;
                unid_subgraph
            }
        };


        let mut dead_node_ids = HashSet::new();
        let mut cached_node_ids = HashSet::new();
        let mut unid_id_map = HashMap::new();

        // new method
        let mut identified_graph = Graph::new(output_subgraph.timestamp);
        for (old_node_key, old_node) in output_subgraph.nodes.iter() {
            let node = old_node.clone();

            match retry_cache.in_cache(old_node_key).await {
                Ok(true) => {
                    info!("Cached node, dropping");
                    cached_node_ids.insert(old_node_key.to_string());
                    continue
                }
                Err(e) => warn!("Failed to retrieve from cache"),
                _ => (), // cache miss
            };

            let node = match self.attribute_node_key(node.clone()).await {
                Ok(node) => node,
                Err(e) => {
                    warn!("Failed to attribute node_key with: {}", e);
                    dead_node_ids.insert(node.clone_node_key());
                    attribution_failure = true;
                    continue;
                }
            };
            unid_id_map.insert(old_node_key.to_owned(), node.clone_node_key());
            identified_graph.add_node(node);
        }


        println!("PRE: identified_graph.edges.len() {}", identified_graph.edges.len());


        for (old_key, edge_list) in output_subgraph.edges.iter() {
            if dead_node_ids.contains(old_key) { continue; };

            for edge in &edge_list.edges {
                let from_key = unid_id_map.get(&edge.from);
                let to_key = unid_id_map.get(&edge.to);

                let (from_key, to_key) = match (from_key, to_key) {
                    (Some(from_key), Some(to_key)) => (from_key, to_key),
                    _ => continue
                };

                identified_graph.add_edge(
                    edge.edge_name.to_owned(),
                    from_key.to_owned(),
                    to_key.to_owned(),
                );
            }
        }

        println!("POST: identified_graph.edges.len() {}", identified_graph.edges.len());

        // Remove dead nodes and edges from output_graph
        let dead_node_ids: HashSet<&str> = dead_node_ids.iter().map(String::as_str).collect();

        if identified_graph.is_empty() {
            return OutputEvent::new(Completion::Error(
                Arc::new(
                    (|| {bail!("All nodes failed to identify"); Ok(())})().unwrap_err()
                )
            ))
        }

        let identities: Vec<_> = unid_id_map.keys().cloned().collect();

        let mut completed = if !dead_node_ids.is_empty() || attribution_failure {
            OutputEvent::new(
                Completion::Partial(
                    (GeneratedSubgraphs::new(vec![identified_graph]), Arc::new(
                        (|| {bail!("Failed"); Ok(())})().unwrap_err()
                    ))
                )
            )
        } else {
            OutputEvent::new(Completion::Total(GeneratedSubgraphs::new(vec![identified_graph])))
        };

        identities.into_iter().for_each(|identity| completed.add_identity(identity));

        completed

//        if !dead_node_ids.is_empty() || attribution_failure {
//            bail!("Some node keys failed to ID")
//        }
//
//        Ok(())
    }
}


pub fn handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    _handler(event, ctx, false)
}

pub fn retry_handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    _handler(event, ctx, true)
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphSerializer {
    proto: Vec<u8>,
}

impl CompletionEventSerializer for SubgraphSerializer {
    type CompletedEvent = GeneratedSubgraphs;
    type Output = Vec<u8>;
    type Error = failure::Error;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        let mut subgraph = Graph::new(
            0
        );

        let mut pre_nodes = 0;
        let mut pre_edges = 0;
        for completed_event in completed_events {
            for sg in completed_event.subgraphs.iter() {
                pre_nodes += sg.nodes.len();
                pre_edges += sg.edges.len();
                subgraph.merge(sg);
            }
        }

        if subgraph.is_empty() {
            warn!(
                concat!(
                "Output subgraph is empty. Serializing to empty vector.",
                "pre_nodes: {} pre_edges: {}"
                ),
                pre_nodes,
                pre_edges,
            );
            return Ok(vec![])
        }

        info!(
            "Serializing {} nodes {} edges. Down from {} nodes {} edges.",
            subgraph.nodes.len(),
            subgraph.edges.len(),
            pre_nodes,
            pre_edges,
        );

        let subgraphs = GeneratedSubgraphs { subgraphs: vec![subgraph] };

        self.proto.clear();

        prost::Message::encode(&subgraphs, &mut self.proto)?;


        let mut compressed = Vec::with_capacity(self.proto.len());
        let mut proto = Cursor::new(&self.proto);
        zstd::stream::copy_encode(&mut proto, &mut compressed, 4)?;

        Ok(vec![compressed])
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZstdProtoDecoder;

impl<E> PayloadDecoder<E> for ZstdProtoDecoder
    where E: Message + Default
{
    fn decode(&mut self, body: Vec<u8>) -> Result<E, Box<dyn std::error::Error>>
        where E: Message + Default,
    {
        let mut decompressed = Vec::new();

        let mut body = Cursor::new(&body);

        zstd::stream::copy_decode(&mut body, &mut decompressed)?;

        Ok(E::decode(decompressed)?)
    }
}


fn time_based_key_fn(_event: &[u8]) -> String {
    info!("event length {}", _event.len());
    let cur_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let cur_day = cur_ms - (cur_ms % 86400);

    format!(
        "{}/{}-{}",
        cur_day, cur_ms, uuid::Uuid::new_v4()
    )
}

fn map_sqs_message(event: aws_lambda_events::event::sqs::SqsMessage) -> rusoto_sqs::Message {
    rusoto_sqs::Message {
        attributes: Some(event.attributes),
        body: event.body,
        md5_of_body: event.md5_of_body,
        md5_of_message_attributes: event.md5_of_message_attributes,
        message_attributes: None,
        message_id: event.message_id,
        receipt_handle: event.receipt_handle,
    }
}

fn _handler(event: SqsEvent, ctx: Context, should_default: bool) -> Result<(), HandlerError> {
    info!("Handling event");

    let mut initial_events: HashSet<String> = event.records
        .iter()
        .map(|event| event.message_id.clone().unwrap())
        .collect();

    info!("Initial Events {:?}", initial_events);

    let (tx, rx) = std::sync::mpsc::sync_channel(10);


    std::thread::spawn(move || {
        tokio_compat::run_std(
            async move {
                let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");
                info!("Queue Url: {}", queue_url);
                let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");
                let cache_address = {
                    let generic_event_cache_addr = std::env::var("GENERIC_EVENT_CACHE_ADDR").expect("GENERIC_EVENT_CACHE_ADDR");
                    let generic_event_cache_port = std::env::var("GENERIC_EVENT_CACHE_PORT").expect("GENERIC_EVENT_CACHE_PORT");

                    format!(
                        "redis://{}:{}/",
                        generic_event_cache_addr,
                        generic_event_cache_port,
                    )
                };

                let bucket = bucket_prefix + "-unid-subgraphs-generated-bucket";
                info!("Output events to: {}", bucket);
                let region = {
                    let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
                    Region::from_str(&region_str).expect("Region error")
                };

                let dynamo = DynamoDbClient::new(region.clone());

                let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));

                let dynamo = DynamoDbClient::new(region.clone());
                let dyn_session_db = SessionDb::new(
                    dynamo.clone(),
                    "dynamic_session_table",
                );
                let dyn_mapping_db = DynamicMappingDb::new(DynamoDbClient::new(region.clone()));
                let asset_identifier = AssetIdentifier::new(asset_id_db);

                let dyn_node_identifier = DynamicNodeIdentifier::new(
                    asset_identifier,
                    dyn_session_db,
                    dyn_mapping_db,
                    should_default,
                );

                let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));

                let asset_identifier = AssetIdentifier::new(asset_id_db);

                let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));
                let node_identifier = NodeIdentifier::new(
                    asset_id_db,
                    dyn_node_identifier,
                    asset_identifier,
                    dynamo.clone(),
                    should_default,
                );


                let cache = RedisCache::new(cache_address.to_owned()).expect("Could not create redis client");

                info!("SqsCompletionHandler");

                let finished_tx = tx.clone();
                let sqs_completion_handler = SqsCompletionHandlerActor::new(
                    SqsCompletionHandler::new(
                        SqsClient::new(region.clone()),
                        queue_url.to_string(),
                        SubgraphSerializer { proto: Vec::with_capacity(1024) },
                        S3EventEmitter::new(
                            S3Client::new(region.clone()),
                            bucket.to_owned(),
                            time_based_key_fn,
                        ),
                        CompletionPolicy::new(
                            1000, // Buffer up to 1000 messages
                            Duration::from_secs(30), // Buffer for up to 30 seconds
                        ),
                        move |_self_actor, result: Result<String, String>| {
                            match result {
                                Ok(worked) => {
                                    info!("Handled an event, which was successfully deleted: {}", &worked);
                                    tx.send(worked).unwrap();
                                }
                                Err(worked) => {
                                    info!("Handled an initial_event, though we failed to delete it: {}", &worked);
                                    tx.send(worked).unwrap();
                                }
                            }
                        },
                    )
                );


                info!("Defining consume policy");
                let consume_policy = ConsumePolicy::new(
                    ctx, // Use the Context.deadline from the lambda_runtime
                    Duration::from_secs(10), // Stop consuming when there's 2 seconds left in the runtime
                    3, // If we get 3 empty receives in a row, stop consuming
                );

                info!("Defining consume policy");
                let (shutdown_tx, shutdown_notify) = tokio::sync::oneshot::channel();

                info!("SqsConsumer");
                let sqs_consumer = SqsConsumerActor::new(
                    SqsConsumer::new(
                        SqsClient::new(region.clone()),
                        queue_url.clone(),
                        consume_policy,
                        sqs_completion_handler.clone(),
                        shutdown_tx,
                    )
                );

                info!("EventProcessors");
                let event_processors: Vec<_> = (0..10)
                    .map(|_| {
                        EventProcessorActor::new(EventProcessor::new(
                            sqs_consumer.clone(),
                            sqs_completion_handler.clone(),
                            node_identifier.clone(),
                            S3PayloadRetriever::new(S3Client::new(region.clone()), ZstdProtoDecoder::default()),
                            cache.clone()
                        ))
                    })
                    .collect();

                info!("Start Processing");

                futures::future::join_all(event_processors.iter().map(|ep| ep.start_processing())).await;

                let mut proc_iter = event_processors.iter().cycle();
                for event in event.records {
                    let next_proc = proc_iter.next().unwrap();
                    next_proc.process_event(
                        map_sqs_message(event)
                    ).await;
                }

                info!("Waiting for shutdown notification");

                // Wait for the consumers to shutdown
                let _ = shutdown_notify.await;
                info!("Consumer shutdown");
                finished_tx.send("Completed".to_owned()).unwrap();
            });
    });

    info!("Checking acks");
    for r in &rx {
        info!("Acking event: {}", &r);
        initial_events.remove(&r);
        if r == "Completed" {
            let r = rx.recv_timeout(Duration::from_millis(100));
            if let Ok(r) = r {
                initial_events.remove(&r);
            }
            // If we're done go ahead and try to clear out any remaining
            while let Ok(r) = rx.try_recv() {
                initial_events.remove(&r);
            }
            break;
        }
    }

    info!("Completed execution");

    if initial_events.is_empty() {
        info!("Successfully acked all initial events");
        Ok(())
    } else {
        Err(lambda::error::HandlerError::from("Failed to ack all initial events"))
    }
}


//fn _handler(event: SqsEvent, ctx: Context, should_default: bool) -> Result<(), HandlerError> {
//    let region = {
//        let region_str = env::var("AWS_REGION").expect("AWS_REGION");
//        Region::from_str(&region_str).expect("Invalid region")
//    };
//
//    let dynamo = DynamoDbClient::new(region.clone());
//
//    let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));
//
//    let dynamo = DynamoDbClient::new(region.clone());
//    let dyn_session_db = SessionDb::new(
//        dynamo.clone(),
//        "dynamic_session_table",
//    );
//    let dyn_mapping_db = DynamicMappingDb::new(DynamoDbClient::new(region.clone()));
//    let asset_identifier = AssetIdentifier::new(asset_id_db);
//
//    let dyn_node_identifier = DynamicNodeIdentifier::new(
//        asset_identifier,
//        dyn_session_db,
//        dyn_mapping_db,
//        should_default,
//    );
//
//    let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));
//
//    let asset_identifier = AssetIdentifier::new(asset_id_db);
//
//    let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));
//    let handler = NodeIdentifier::new(
//        asset_id_db,
//        dyn_node_identifier,
//        asset_identifier,
//        dynamo.clone(),
//        upload_identified_graphs,
//        should_default,
//    );
//
//    let sqs_client = Arc::new(SqsClient::new(region.clone()));
//
//    info!("Creating s3_client");
//    let s3_client = Arc::new(S3Client::new(region.clone()));
//
//    info!("Creating retriever");
//    let retriever = S3EventRetriever::new(
//        s3_client,
//        |d| {
//            info!("Parsing: {:?}", d);
//            events_from_s3_sns_sqs(d)
//        },
//        ZstdProtoDecoder {},
//    );
//
//    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");
//
//    info!("Creating sqs_completion_handler");
//    let sqs_completion_handler = NopSqsCompletionHandler::new(queue_url);
//
//    let mut sqs_service = SqsService::new(retriever, handler, sqs_completion_handler);
//
//    info!("Handing off event");
//    sqs_service.run(event, ctx)?;
//
//    Ok(())
//}
//
