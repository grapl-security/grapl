#![allow(dead_code)]

extern crate aws_lambda_events;
extern crate base58;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate hex;
#[macro_use]
extern crate hmap;
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

use rusoto_sqs::Sqs;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryFrom;
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
use graph_descriptions::file::FileState;
use graph_descriptions::graph_description::*;
use graph_descriptions::graph_description::host::*;
use graph_descriptions::graph_description::node::WhichNode;
use graph_descriptions::ip_connection::IpConnectionState;
use graph_descriptions::network_connection::NetworkConnectionState;
use graph_descriptions::process::ProcessState;
use graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use lambda::Context;
use lambda::error::HandlerError;
use prost::Message;
use rusoto_core::{Region, HttpClient};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
use rusoto_s3::S3Client;
use rusoto_sqs::{SqsClient, SendMessageRequest};
use sha2::Digest;
use sqs_lambda::cache::{Cache, CacheResponse, NopCache, Cacheable};
use sqs_lambda::completion_event_serializer::CompletionEventSerializer;
use sqs_lambda::event_decoder::PayloadDecoder;
use sqs_lambda::s3_event_emitter::S3EventEmitter;
use sqs_lambda::event_handler::{Completion, EventHandler, OutputEvent};
use sqs_lambda::event_processor::{EventProcessor, EventProcessorActor};
use sqs_lambda::event_retriever::S3PayloadRetriever;
use sqs_lambda::redis_cache::RedisCache;
use sqs_lambda::sqs_completion_handler::{CompletionPolicy, SqsCompletionHandler, SqsCompletionHandlerActor};
use sqs_lambda::sqs_consumer::{ConsumePolicy, SqsConsumer, SqsConsumerActor};

use assetdb::{AssetIdDb, AssetIdentifier};
use async_trait::async_trait;
use dynamic_sessiondb::{DynamicMappingDb, DynamicNodeIdentifier};
use sessiondb::SessionDb;
use sessions::UnidSession;

use crate::graph_descriptions::node::NodeT;
use std::fmt::Debug;
use sqs_lambda::local_service::local_service;
use aws_lambda_events::event::s3::{S3EventRecord, S3Event, S3UserIdentity, S3RequestParameters, S3Entity, S3Bucket, S3Object};
use sqs_lambda::local_sqs_service::local_sqs_service;
use chrono::Utc;

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
            $x
        .await
    }};
}

pub mod assetdb;
pub mod dynamic_sessiondb;

pub mod sessiondb;
pub mod sessions;

#[derive(Clone)]
struct NodeIdentifier<D, CacheT, CacheErr>
    where
        D: DynamoDb + Clone + Send + Sync + 'static,
        CacheT: Cache<CacheErr> + Clone + Send + Sync + 'static,
        CacheErr: Debug + Clone + Send + Sync + 'static,
{
    asset_mapping_db: AssetIdDb<D>,
    dynamic_identifier: DynamicNodeIdentifier<D>,
    asset_identifier: AssetIdentifier<D>,
    node_id_db: D,
    should_default: bool,
    cache: CacheT,
    region: Region,
    _p: std::marker::PhantomData<CacheErr>,
}

impl<D, CacheT, CacheErr> NodeIdentifier<D, CacheT, CacheErr>
    where
        D: DynamoDb + Clone + Send + Sync + 'static,
        CacheT: Cache<CacheErr> + Clone + Send + Sync + 'static,
        CacheErr: Debug + Clone + Send + Sync + 'static,

{
    pub fn new(
        asset_mapping_db: AssetIdDb<D>,
        dynamic_identifier: DynamicNodeIdentifier<D>,
        asset_identifier: AssetIdentifier<D>,
        node_id_db: D,
        should_default: bool,
        cache: CacheT,
        region: Region,
    ) -> Self {
        Self {
            asset_mapping_db,
            dynamic_identifier,
            asset_identifier,
            node_id_db,
            should_default,
            cache,
            region,
            _p: std::marker::PhantomData,
        }
    }

    async fn attribute_node_key(&self, node: Node) -> Result<Node, Error> {
        let unid = into_unid_session(&node)?;

        match node.which_node {
            Some(WhichNode::ProcessNode(mut process_node)) => {
                info!("Attributing ProcessNode: {}", process_node.process_id);
                let unid = match unid {
                    Some(unid) => unid,
                    None => bail!("Could not identify ProcessNode")
                };
                let session_db = SessionDb::new(self.node_id_db.clone(), "process_history_table");
                let node_key = session_db.handle_unid_session(unid, self.should_default).await?;

                info!(
                    "Mapped Process {:?} to {}",
                    process_node,
                    &node_key,
                );
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
                let session_db = SessionDb::new(self.node_id_db.clone(), "inbound_connection_history_table");
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
            let (is_creation, timestamp) = match (
                node.created_timestamp != 0,
                node.last_seen_timestamp != 0,
                node.terminated_timestamp != 0,
            ) {
                (true, _, _) => (true, node.created_timestamp),
                (_, _, true) => (false, node.terminated_timestamp),
                (_, true, _) => (false, node.last_seen_timestamp),
                _ => bail!("At least one timestamp must be set")
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
        }

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
        }
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
) -> Result<Graph, (Error, Graph)> {
    info!("Attributing asset ids");
    let mut dead_nodes = HashSet::new();
    let mut output_graph = Graph::new(unid_graph.timestamp);
    output_graph.edges = unid_graph.edges;

    let node_asset_ids: HashMap<String, String> = HashMap::new();
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
            _ => ()
        }

        let asset_id = asset_identifier.attribute_asset_id(
            &node,
        ).await;

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

#[async_trait]
impl<D, CacheT, CacheErr> EventHandler for NodeIdentifier<D, CacheT, CacheErr>
    where
        D: DynamoDb + Clone + Send + Sync + 'static,
        CacheT: Cache<CacheErr> + Clone + Send + Sync + 'static,
        CacheErr: Debug + Clone + Send + Sync + 'static,

{
    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = GeneratedSubgraphs;
    type Error = sqs_lambda::error::Error<Arc<failure::Error>>;

    async fn handle_event(&mut self, subgraphs: GeneratedSubgraphs) -> OutputEvent<Self::OutputEvent, Self::Error> {
        warn!("node-identifier.handle_event");
        let region = self.region.clone();

        let mut attribution_failure = None;

        info!("Handling raw event");

        if subgraphs.subgraphs.is_empty() {
            warn!("Received empty unid subgraph");
            return OutputEvent::new(Completion::Total(GeneratedSubgraphs::new(vec![])));
        }

        let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));
//        let dynamo = DynamoDbClient::new(region.clone());

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
            return OutputEvent::new(Completion::Error(
                sqs_lambda::error::Error::ProcessingError(
                    Arc::new(e.into()))
            )
            );
        }

        // Map all host_ids into asset_ids. This has to happen before node key
        // identification.
        // If there is a failure, we'll mark this execute as failed, but continue
        // with whatever subgraph has succeeded

        let output_subgraph = match attribute_asset_ids(&self.asset_identifier, unid_subgraph).await {
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
                    attribution_failure = Some(e);
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

        info!("POST: identified_graph.edges.len() {}", identified_graph.edges.len());

        // Remove dead nodes and edges from output_graph
        let dead_node_ids: HashSet<&str> = dead_node_ids.iter().map(String::as_str).collect();

        if identified_graph.is_empty() {
            return OutputEvent::new(Completion::Error(
                sqs_lambda::error::Error::ProcessingError(
                    Arc::new(
                        (|| {
                            bail!("All nodes failed to identify");
                            Ok(())
                        })().unwrap_err()
                    )
                ))
            );
        }

        let identities: Vec<_> = unid_id_map.keys().cloned().collect();

        let mut completed = if !dead_node_ids.is_empty() || attribution_failure.is_some() {
            info!("Partial Success, identified {} nodes", identities.len());
            OutputEvent::new(
                Completion::Partial(
                    (
                        GeneratedSubgraphs::new(vec![identified_graph]),
                        sqs_lambda::error::Error::ProcessingError(Arc::new(
                            attribution_failure.unwrap()
                        ))
                    )
                )
            )
        } else {
            info!("Identified all nodes");
            OutputEvent::new(Completion::Total(GeneratedSubgraphs::new(vec![identified_graph])))
        };

        completed
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
    type Error = sqs_lambda::error::Error<Arc<failure::Error>>;

    fn serialize_completed_events(
        &mut self,
        completed_events: &[Self::CompletedEvent],
    ) -> Result<Vec<Self::Output>, Self::Error> {
        if completed_events.is_empty() {
            warn!("No events to serialize");
            return Ok(Vec::new());
        }
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
            return Ok(vec![]);
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

        prost::Message::encode(&subgraphs, &mut self.proto)
            .map(Arc::new)
            .map_err(|e| {
                sqs_lambda::error::Error::EncodeError(e.to_string())
            })?;

        let mut compressed = Vec::with_capacity(self.proto.len());
        let mut proto = Cursor::new(&self.proto);
        zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
            .map(Arc::new)
            .map_err(|e| {
                sqs_lambda::error::Error::EncodeError(e.to_string())
            })?;
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

    let completed_tx = tx.clone();

    std::thread::spawn(move || {
        tokio_compat::run_std(
                async move {
                let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");
                info!("Queue Url: {}", queue_url);
                let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");
                let cache_address = {
                    let retry_identity_cache_addr = std::env::var("RETRY_IDENTITY_CACHE_ADDR").expect("RETRY_IDENTITY_CACHE_ADDR");
                    let retry_identity_cache_port = std::env::var("RETRY_IDENTITY_CACHE_PORT").expect("RETRY_IDENTITY_CACHE_PORT");

                    format!(
                        "{}:{}",
                        retry_identity_cache_addr,
                        retry_identity_cache_port,
                    )
                };

                let bucket = bucket_prefix + "-subgraphs-generated-bucket";
                info!("Output events to: {}", bucket);
                let region = {
                    let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
                    Region::from_str(&region_str).expect("Region error")
                };
                let cache = RedisCache::new(cache_address.to_owned()).await.expect("Could not create redis client");

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
                let node_identifier
                    : NodeIdentifier<_, _, sqs_lambda::error::Error<Arc<failure::Error>>>
                    = NodeIdentifier::new(
                    asset_id_db,
                    dyn_node_identifier,
                    asset_identifier,
                    dynamo.clone(),
                    should_default,
                    cache.clone(),
                    region.clone(),
                );

                let initial_messages: Vec<_> = event.records
                    .into_iter()
                    .map(map_sqs_message)
                    .collect();

                sqs_lambda::sqs_service::sqs_service(
                    queue_url,
                    initial_messages,
                    bucket,
                    ctx,
                    S3Client::new(region.clone()),
                    SqsClient::new(region.clone()),
                    ZstdProtoDecoder::default(),
                    SubgraphSerializer { proto: Vec::with_capacity(1024) },
                    node_identifier,
                    cache.clone(),
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
                    move |_, _| async move { Ok(()) }
                ).await;

                completed_tx.clone().send("Completed".to_owned()).unwrap();
        })
    });

    info!("Checking acks");
    for r in &rx {
        info!("Acking event: {}", &r);
        initial_events.remove(&r);
        if r == "Completed" {
            // If we're done go ahead and try to clear out any remaining
            while let Ok(r) = rx.recv_timeout(Duration::from_millis(100)) {
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

fn init_sqs_client() -> SqsClient
{
    info!("Connecting to local us-east-1 http://sqs.us-east-1.amazonaws.com:9324");

    SqsClient::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        rusoto_credential::StaticProvider::new_minimal(
            "dummy_sqs".to_owned(),
            "dummy_sqs".to_owned(),
        ),
        Region::Custom {
            name: "us-east-1".to_string(),
            endpoint: "http://sqs.us-east-1.amazonaws.com:9324".to_string(),
        }
    )
}

fn init_s3_client() -> S3Client
{
    info!("Connecting to local http://s3:9000");
    S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        rusoto_credential::StaticProvider::new_minimal(
            "minioadmin".to_owned(),
            "minioadmin".to_owned(),
        ),
        Region::Custom {
            name: "locals3".to_string(),
            endpoint: "http://s3:9000".to_string(),
        },
    )
}


fn init_dynamodb_client() -> DynamoDbClient
{
    info!("Connecting to local http://dynamo:9000");
    DynamoDbClient::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        rusoto_credential::StaticProvider::new_minimal(
            "dummy_cred_aws_access_key_id".to_owned(),
            "dummy_cred_aws_secret_access_key".to_owned(),
        ),
        Region::Custom {
            name: "us-west-2".to_string(),
            endpoint: "http://dynamo:8000".to_string(),
        },
    )
}

#[derive(Clone, Default)]
pub struct LocalCache {
    inner_map: HashSet<Vec<u8>>,
}

#[async_trait]
impl<E> Cache<E> for LocalCache
    where
        E: Debug + Clone + Send + Sync + 'static,
{
    async fn get<CA: Cacheable + Send + Sync + 'static>(&mut self, cacheable: CA)
        -> Result<CacheResponse, sqs_lambda::error::Error<E>>
    {
        match self.inner_map.contains(&cacheable.identity()) {
            true => Ok(CacheResponse::Hit),
            false => Ok(CacheResponse::Miss),
        }
    }

    async fn store(&mut self, identity: Vec<u8>)
        -> Result<(), sqs_lambda::error::Error<E>>
    {
        self.inner_map.insert(identity);
        Ok(())
    }
}


pub async fn local_handler(should_default: bool) -> Result<(), HandlerError> {
    let cache = LocalCache::default();

    info!("region");
    let region = Region::Custom {
        name: "dynamo".to_string(),
        endpoint: "http://dynamo:8222".to_string(),
    };

    info!("asset_id_db");
    let asset_id_db = AssetIdDb::new(init_dynamodb_client());

    info!("dynamo");
    let dynamo = init_dynamodb_client();
    info!("dyn_session_db");
    let dyn_session_db = SessionDb::new(
        dynamo.clone(),
        "dynamic_session_table",
    );
    info!("dyn_mapping_db");
    let dyn_mapping_db = DynamicMappingDb::new(init_dynamodb_client());
    info!("asset_identifier");
    let asset_identifier = AssetIdentifier::new(asset_id_db);

    info!("dyn_node_identifier");
    let dyn_node_identifier = DynamicNodeIdentifier::new(
        asset_identifier,
        dyn_session_db,
        dyn_mapping_db,
        should_default,
    );

    info!("asset_id_db");
    let asset_id_db = AssetIdDb::new(init_dynamodb_client());

    info!("asset_identifier");
    let asset_identifier = AssetIdentifier::new(asset_id_db);

    info!("asset_id_db");
    let asset_id_db = AssetIdDb::new(init_dynamodb_client());

    info!("node_identifier");
    let node_identifier
        : NodeIdentifier<_, _, sqs_lambda::error::Error<Arc<failure::Error>>>
        = NodeIdentifier::new(
            asset_id_db,
            dyn_node_identifier,
            asset_identifier,
            dynamo.clone(),
            should_default,
            cache.clone(),
            region.clone(),
    );

    let queue_url = if should_default {
        "http://sqs.us-east-1.amazonaws.com:9324/queue/node-identifier-queue"
    } else {
        "http://sqs.us-east-1.amazonaws.com:9324/queue/node-identifier-retry-queue"
    };

    local_sqs_service(
        queue_url,
        "local-grapl-subgraphs-generated-bucket",
        Context {
            deadline: Utc::now().timestamp_millis() + 10_000,
            ..Default::default()
        },
        init_s3_client(),
        init_sqs_client(),
        ZstdProtoDecoder::default(),
        SubgraphSerializer { proto: Vec::with_capacity(1024) },
        node_identifier,
        LocalCache::default(),
        |_, event_result | {dbg!(event_result);},
        move |bucket, key| async move {
            let output_event = S3Event {
                records: vec![
                    S3EventRecord {
                        event_version: None,
                        event_source: None,
                        aws_region: None,
                        event_time: chrono::Utc::now(),
                        event_name: None,
                        principal_id: S3UserIdentity { principal_id: None },
                        request_parameters: S3RequestParameters { source_ip_address: None },
                        response_elements: Default::default(),
                        s3: S3Entity {
                            schema_version: None,
                            configuration_id: None,
                            bucket: S3Bucket {
                                name: Some(bucket),
                                owner_identity: S3UserIdentity { principal_id: None },
                                arn: None
                            },
                            object: S3Object {
                                key: Some(key),
                                size: 0,
                                url_decoded_key: None,
                                version_id: None,
                                e_tag: None,
                                sequencer: None
                            }
                        }
                    }
                ]
            };

            let sqs_client = init_sqs_client();

            // publish to SQS
            sqs_client.send_message(
                SendMessageRequest {
                    message_body: serde_json::to_string(&output_event)
                        .expect("failed to encode s3 event"),
                    queue_url: "http://sqs.us-east-1.amazonaws.com:9324/queue/graph-merger-queue".to_string(),
                    ..Default::default()
                }
            ).await?;

            Ok(())
        }
    ).await;
    Ok(())
}