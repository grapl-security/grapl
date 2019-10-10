#![allow(unused_variables, unused_imports, dead_code)]

extern crate aws_lambda_events;
extern crate base58;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
#[macro_use]
extern crate hmap;
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;
extern crate lru_time_cache;

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

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::hash::Hash;
use std::io::Cursor;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use base58::ToBase58;
use failure::Error;
use futures::future::Future;
use graph_descriptions::*;
use graph_descriptions::graph_description::*;
use graph_descriptions::graph_description::host::*;
use lambda::Context;
use lambda::error::HandlerError;
use lru_time_cache::LruCache;

use prost::Message;
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
use rusoto_s3::{S3, S3Client};
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};
use sha2::{Digest, Sha256};
use sqs_lambda::EventHandler;
use sqs_lambda::events_from_s3_sns_sqs;
use sqs_lambda::NopSqsCompletionHandler;
use sqs_lambda::S3EventRetriever;
use sqs_lambda::SqsService;
use sqs_lambda::ZstdProtoDecoder;
use stopwatch::Stopwatch;

use assetdb::{AssetIdDb, AssetIdentifier};
use sessiondb::SessionDb;
use sessions::{shave_int, UnidSession};
use dynamic_sessiondb::{DynamicNodeIdentifier, DynamicMappingDb};

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

        $x.with_timeout(Duration::from_secs(2)).sync()
    }};
}

pub mod assetdb;
pub mod dynamic_sessiondb;
pub mod retry_cache;
pub mod sessiondb;
pub mod sessions;

#[derive(Clone)]
struct NodeIdentifier<D, F>
where
    D: DynamoDb + Clone,
    F: (Fn(GraphDescription) -> Result<(), Error>) + Clone,
{
    asset_mapping_db: AssetIdDb<D>,
    dynamic_identifier: DynamicNodeIdentifier<D>,
    asset_identifier: AssetIdentifier<D>,
    node_id_db: D,
    should_default: bool,
    output_handler: F,
}

impl<D, F> NodeIdentifier<D, F>
where
    D: DynamoDb + Clone,
    F: (Fn(GraphDescription) -> Result<(), Error>) + Clone,
{
    pub fn new(
        asset_mapping_db: AssetIdDb<D>,
        dynamic_identifier: DynamicNodeIdentifier<D>,
        asset_identifier: AssetIdentifier<D>,
        node_id_db: D,
        output_handler: F,
        should_default: bool,
    ) -> Self {
        Self {
            asset_mapping_db,
            dynamic_identifier,
            asset_identifier,
            node_id_db,
            should_default,
            output_handler,
        }
    }
}

fn into_unid_session(node: impl Into<Node>) -> Option<UnidSession> {
    match node.into() {
        Node::ProcessNode(node) => {
            let is_creation = match ProcessState::from(node.state) {
                ProcessState::Created => true,
                _ => false,
            };
            UnidSession {
                pseudo_key: format!(
                    "{}{}",
                    node.get_asset_id().expect("Missing asset id"),
                    node.process_id
                ),
                timestamp: node.timestamp(),
                is_creation,
            }
            .into()
        }
        Node::FileNode(node) => {
            let is_creation = match FileState::from(node.state) {
                FileState::Created => true,
                _ => false,
            };
            // TODO: Hash the path
            let key = &node.file_path;
            UnidSession {
                pseudo_key: format!("{}{}", node.get_asset_id().expect("Missing asset id"), key),
                timestamp: node.timestamp(),
                is_creation,
            }
            .into()
        }
        Node::OutboundConnectionNode(node) => {
            let is_creation = match ConnectionState::from(node.state) {
                ConnectionState::Created => true,
                _ => false,
            };
            UnidSession {
                pseudo_key: format!(
                    "{}{}outbound",
                    node.get_asset_id().expect("Missing asset id"),
                    node.port
                ),
                timestamp: node.timestamp(),
                is_creation,
            }
            .into()
        }
        Node::InboundConnectionNode(node) => {
            let is_creation = match ConnectionState::from(node.state) {
                ConnectionState::Created => true,
                _ => false,
            };
            UnidSession {
                pseudo_key: format!(
                    "{}{}inbound",
                    node.get_asset_id().expect("Missing asset id"),
                    node.port
                ),
                timestamp: node.timestamp(),
                is_creation,
            }
            .into()
        }
        Node::DynamicNode(node) => {
            None
        },
        Node::IpAddressNode(node) => None,
        Node::AssetNode(node) => unimplemented!(),
    }
}

fn remove_dead_nodes(graph: &mut GraphDescription, dead_nodes: &HashSet<impl Deref<Target = str>>) {
    for dead_node in dead_nodes {
        graph.nodes.remove(dead_node.deref());
        graph.edges.remove(dead_node.deref());
    }
}

fn remove_dead_edges(graph: &mut GraphDescription) {
    let edges = &mut graph.edges;
    let nodes = &graph.nodes;
    for (node_key, edge_list) in edges.iter_mut() {
        let live_edges: Vec<_> = edge_list
            .edges
            .clone()
            .into_iter()
            .filter(|edge| nodes.contains_key(&edge.to) && nodes.contains_key(&edge.from))
            .collect();

        *edge_list = EdgeList { edges: live_edges };
    }
}

fn remap_edges(graph: &mut GraphDescription, unid_id_map: &HashMap<String, String>) {
    for (node_key, edge_list) in graph.edges.iter_mut() {
        for edge in edge_list.edges.iter_mut() {
            let from = match unid_id_map.get(&edge.from) {
                Some(from) => from,
                None => continue
            };

            let to = match unid_id_map.get(&edge.to) {
                Some(to) => to,
                None => continue
            };

            *edge = EdgeDescription {
                from: from.to_owned(),
                to: to.to_owned(),
                edge_name: edge.edge_name.clone(),
            };
        }
    }
}

fn remap_nodes(graph: &mut GraphDescription, unid_id_map: &HashMap<String, String>) {
    let mut nodes = HashMap::with_capacity(graph.nodes.len());

    for (node_key, node) in graph.nodes.iter_mut() {
        // DynamicNodes are identified in-place
        if let Node::DynamicNode(n) = node.clone().which() {
            let old_node = nodes.insert(node.get_key().to_owned(), node.clone());
            if let Some(ref old_node) = old_node {
                nodes
                    .get_mut(node.get_key())
                    .expect("New key not in map")
                    .merge(old_node);
            }
            continue
        }
        let new_key = unid_id_map.get(node.get_key()).expect("node_key mapping did not exist");
        node.set_key(new_key.to_owned());

        // We may have actually had nodes with different unid node_keys that map to the
        // same node_key. Therefor we must merge any nodes when there is a collision.
        let old_node = nodes.insert(new_key.to_owned(), node.clone());
        if let Some(ref old_node) = old_node {
            nodes
                .get_mut(new_key)
                .expect("New key not in map")
                .merge(old_node);
        }
    }
    graph.nodes = nodes;
}

fn create_asset_id_mappings(
    assetid_db: &AssetIdDb<impl DynamoDb>,
    unid_graph: &GraphDescription,
) -> Result<(), Error> {
    for node in unid_graph.nodes.values() {
        let ids = match node.clone().which() {
            Node::ProcessNode(node) => (node.asset_id, node.hostname, node.host_ip),
            Node::OutboundConnectionNode(node) => (node.asset_id, node.hostname, node.host_ip),
            Node::FileNode(node) => (node.asset_id, node.hostname, node.host_ip),
            _ => continue,
        };

        match ids {
            (Some(asset_id), Some(hostname), Some(host_ip)) => {
                info!("Creating asset id {} mapping for: {}", asset_id, hostname);
                info!("Creating asset id mapping for: ip");
                assetid_db.create_mapping(
                    &HostId::AssetId(asset_id.clone()),
                    hostname,
                    node.get_timestamp(),
                )?;

                assetid_db.create_mapping(
                    &HostId::AssetId(asset_id),
                    host_ip.clone(),
                    node.get_timestamp(),
                )?;
            }
            (Some(asset_id), Some(hostname), _) => {
                info!("Creating asset id {} mapping for: {}", asset_id, hostname);

                assetid_db.create_mapping(
                    &HostId::AssetId(asset_id),
                    hostname,
                    node.get_timestamp(),
                )?;
            }
            (Some(asset_id), _, Some(host_ip)) => {
                info!("Creating asset id mapping for: ip");
                assetid_db.create_mapping(
                    &HostId::AssetId(asset_id),
                    host_ip.clone(),
                    node.get_timestamp(),
                )?;
            }
            _ => continue,
        };
    }

    Ok(())
}


// Takes a GraphDescription, attributes all nodes with an asset id
// When atribution fails, attribution continues, but the Graph returned will contain
// only the nodes that were successful
// Edges will also be fixed up
fn attribute_asset_ids(
    asset_identifier: &AssetIdentifier<impl DynamoDb>,
    unid_graph: GraphDescription,
) -> Result<GraphDescription, GraphDescription> {
    info!("Attributing asset ids");
    let mut dead_nodes = HashSet::new();
    let mut output_graph = GraphDescription::new(unid_graph.timestamp);
    output_graph.edges = unid_graph.edges;


    let node_asset_ids: HashMap<String, String> = HashMap::new();

    for node in unid_graph.nodes.values() {
        match node.clone().which() {
            Node::IpAddressNode(n) => {
                output_graph.add_node(n);
                continue
            },
            Node::DynamicNode(n) => {
                if !n.requires_asset_identification() {
                    output_graph.add_node(n);
                    continue
                }
            }
            _ => ()
        }

        let asset_id = asset_identifier.attribute_asset_id(
            node.clone(),
        );

        let asset_id = match asset_id {
            Ok(asset_id) => asset_id,
            Err(e) => {
                warn!("Failed to attribute to asset id: {:?} {}", node, e);
                dead_nodes.insert(node.get_key().to_owned());
                continue
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

impl<D, F> EventHandler<GeneratedSubgraphs> for NodeIdentifier<D, F>
    where D: DynamoDb + Clone,
          F: (Fn(GraphDescription) -> Result<(), Error>) + Clone,
{
    fn handle_event(&self, subgraphs: GeneratedSubgraphs) -> Result<(), Error> {
        let region = {
            let region_str = env::var("AWS_REGION").expect("AWS_REGION");
            Region::from_str(&region_str)?
        };

        let mut attribution_failure = false;

        info!("Handling raw event");

        if subgraphs.subgraphs.is_empty() {
            warn!("Received empty unid subgraph");
            return Ok(());
        }

        let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));
        let dynamo = DynamoDbClient::new(region.clone());
        let dyn_session_db = SessionDb::new(
            dynamo.clone(),
            "dynamic_session_table"
        );
        let dyn_mapping_db = DynamicMappingDb::new(DynamoDbClient::new(region.clone()));

        let retry_cache =
            retry_cache::RetrySessionCache::new("node_id_retry_table", DynamoDbClient::new(region));

        // Merge all of the subgraphs into one subgraph to avoid
        // redundant work
        let unid_subgraph = subgraphs.subgraphs.into_iter().fold(
            GraphDescription::new(0),
            |mut total_graph, subgraph| {
                total_graph.merge(&subgraph);
                total_graph
            },
        );

        info!(
            "unid_subgraph: {} nodes {} edges",
            unid_subgraph.nodes.len(),
            unid_subgraph.edges.len()
        );

        // Create any implicit asset id mappings
        if let Err(e) = create_asset_id_mappings(&asset_id_db, &unid_subgraph) {
            error!("Asset mapping creation failed with {}", e);
            bail!(e)
        }

        // Map all host_ids into asset_ids. This has to happen before node key
        // identification.
        // If there is a failure, we'll mark this execute as failed, but continue
        // with whatever subgraph has succeeded

        let output_subgraph = match attribute_asset_ids(&self.asset_identifier, unid_subgraph) {
            Ok(unid_subgraph) => unid_subgraph,
            Err(unid_subgraph) => {
                attribution_failure = true;
                unid_subgraph
            }
        };

        if output_subgraph.is_empty() {
            bail!("No host_ids could be mapped to asset_ids");
        }

        let mut unid_id_map = HashMap::new();

        let mut output_subgraph = match self.dynamic_identifier.attribute_dynamic_nodes(output_subgraph, &mut unid_id_map) {
          Ok(unid_subgraph) => unid_subgraph,
            Err(unid_subgraph) => {
                if unid_subgraph.is_empty() {
                    bail!("Asset attribution failed for all nodes")
                }
                attribution_failure = true;
                unid_subgraph
            }
        };

        let mut cached_node_ids = HashSet::new();

        let unid_sessions: Vec<_> = output_subgraph
            .clone()
            .nodes
            .into_iter()
            .map(|(_, n)| n)
            .map(NodeDescription::which)
            .flat_map(|node| {
                if let Node::IpAddressNode(_) = node {
                    info!("Unid IpAddressNode");
                    return Some((node, None));
                }
                match into_unid_session(node.clone()) {
                    Some(unid) => Some((node, Some(unid))),
                    None => None,
                }
            })
            .filter(|(node, unid)| {
                if let Node::IpAddressNode(_) = node {
                    return true;
                }

                let is_cached = retry_cache
                    .in_cache(node.clone_key())
                    .map_err(|e| warn!("Failed to retrieve from retry_cache: {}", e))
                    .unwrap_or(false);

                if is_cached {
                    cached_node_ids.insert(node.clone_key());
                }

                !is_cached
            })
            .collect();

        info!("Removing dead nodes");
        remove_dead_nodes(&mut output_subgraph, &cached_node_ids);
        info!("Removing dead edges");
        remove_dead_edges(&mut output_subgraph);

        // Mapping from old id to new id

        // Mapping from new id back to old id
        let mut id_unid_map = HashSet::new();
        // IDs for nodes that did not get properly identified
        let mut dead_node_ids = HashSet::new();

        for (node, unid) in unid_sessions {
            let session_id = match node {
                Node::ProcessNode(node) => {
                    let unid = unid.unwrap();
                    let session_db = SessionDb::new(self.node_id_db.clone(), "process_history_table");
                    session_db
                        .handle_unid_session(unid, self.should_default)
                        .map(|sid| (node.clone().node_key, sid))
                        .map_err(|e| (node.node_key, e))
                }
                Node::FileNode(node) => {
                    let unid = unid.unwrap();
                    let session_db = SessionDb::new(self.node_id_db.clone(), "file_history_table");
                    session_db
                        .handle_unid_session(unid, self.should_default)
                        .map(|sid| (node.clone().node_key, sid))
                        .map_err(|e| (node.node_key, e))
                }
                Node::OutboundConnectionNode(node) => {
                    info!("Attributing OutboundConnectionNode");

                    let unid = unid.unwrap();
                    let session_db = SessionDb::new(self.node_id_db.clone(), "outbound_connection_history_table");
                    session_db
                        .handle_unid_session(unid, self.should_default)
                        .map(|sid| (node.clone().node_key, sid))
                        .map_err(|e| (node.node_key, e))
                }
                Node::IpAddressNode(node) => {
                    info!("Attributing IpAddressNode");
                    Ok((node.node_key.clone(), node.node_key))
                }
                _ => {
                    warn!("Unsupported node type");
                    continue;
                }
            };

            match session_id {
                Ok((old_key, session_id)) => {
                    unid_id_map.insert(old_key.clone(), session_id);
                    id_unid_map.insert(old_key);
                }
                Err((failed_key, e)) => {
                    warn!("Node Identification failed with {}", e);
                    dead_node_ids.insert(failed_key);
                }
            }
        }

        // Remove dead nodes and edges from output_graph
        let dead_node_ids: HashSet<&str> = dead_node_ids.iter().map(String::as_str).collect();

        info!("removing dead nodes2");
        remove_dead_nodes(&mut output_subgraph, &dead_node_ids);
        info!("removing dead edges2");
        remove_dead_edges(&mut output_subgraph);

        info!("remapping nodes2");
        remap_nodes(&mut output_subgraph, &unid_id_map);
        info!("remapping edges2");
        remap_edges(&mut output_subgraph, &unid_id_map);

        if output_subgraph.is_empty() {
            bail!("Attribution failed for all nodes");
        }

        upload_identified_graphs(output_subgraph)?;

        id_unid_map.iter().for_each(|old_key| {
            retry_cache
                .put_cache(old_key)
                .map_err(|e| {
                    warn!("Failed to update retry cache: {}", e);
                })
                .ok();
        });

        if !dead_node_ids.is_empty() || attribution_failure {
            bail!("Some node keys failed to ID")
        }

        Ok(())
    }
}

pub fn upload_identified_graphs(subgraph: GraphDescription) -> Result<(), Error> {
    info!("Uploading identified subgraphs: {} nodes {} edges",
        subgraph.nodes.len(),
        subgraph.edges.len(),
    );
    let region = {
        let region_str = env::var("AWS_REGION").expect("AWS_REGION");
        Region::from_str(&region_str)?
    };
    let s3 = S3Client::new(region);

    let mut body = Vec::with_capacity(5000);
    subgraph
        .encode(&mut body)
        .expect("Failed to encode subgraph");

    let mut compressed = Vec::with_capacity(body.len());
    let mut proto = Cursor::new(&body);

    zstd::stream::copy_encode(&mut proto, &mut compressed, 4).expect("compress zstd capnp");

    let mut hasher = Sha256::default();
    hasher.input(&body);

    let key = hasher.result().as_ref().to_base58();

    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

    let bucket = bucket_prefix + "-subgraphs-generated-bucket";
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before epoch")
        .as_secs();

    let day = epoch - (epoch % (24 * 60 * 60));

    let key = format!("{}/{}-{}", day, epoch, key);

    info!("Uploading identified subgraphs to {}", key);
    wait_on!(s3.put_object(rusoto_s3::PutObjectRequest {
        bucket,
        key: key.clone(),
        body: Some(compressed.into()),
        ..Default::default()
    }))?;

    info!("Uploaded identified subgraphs to {}", key);

    Ok(())
}

pub fn handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    _handler(event, ctx, false)
}

pub fn retry_handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    _handler(event, ctx, true)
}

fn _handler(event: SqsEvent, ctx: Context, should_default: bool) -> Result<(), HandlerError> {
    let region = {
        let region_str = env::var("AWS_REGION").expect("AWS_REGION");
        Region::from_str(&region_str).expect("Invalid region")
    };

    let dynamo = DynamoDbClient::new(region.clone());

    let asset_id_db = AssetIdDb::new(DynamoDbClient::new(region.clone()));

    let dynamo = DynamoDbClient::new(region.clone());
    let dyn_session_db = SessionDb::new(
        dynamo.clone(),
        "dynamic_session_table"
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
    let handler = NodeIdentifier::new(
        asset_id_db,
        dyn_node_identifier,
        asset_identifier,
        dynamo.clone(),
        upload_identified_graphs,
        should_default,
    );

    let sqs_client = Arc::new(SqsClient::new(region.clone()));

    info!("Creating s3_client");
    let s3_client = Arc::new(S3Client::new(region.clone()));

    info!("Creating retriever");
    let retriever = S3EventRetriever::new(
        s3_client,
        |d| {
            info!("Parsing: {:?}", d);
            events_from_s3_sns_sqs(d)
        },
        ZstdProtoDecoder {},
    );

    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    info!("Creating sqs_completion_handler");
    let sqs_completion_handler = NopSqsCompletionHandler::new(queue_url);

    let mut sqs_service = SqsService::new(retriever, handler, sqs_completion_handler);

    info!("Handing off event");
    sqs_service.run(event, ctx)?;

    Ok(())
}

