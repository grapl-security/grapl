use std::{collections::{HashMap,
                        HashSet},
          fmt::Debug,
          sync::{Arc,
                 Mutex}};

use async_trait::async_trait;
use dynamic_sessiondb::{DynamicMappingDb,
                        NodeDescriptionIdentifier};
use failure::Error;
use grapl_config::{env_helpers::{s3_event_emitters_from_env,
                                 FromEnv},
                   event_caches};
use grapl_graph_descriptions::graph_description::{GraphDescription,
                                                  IdentifiedGraph,
                                                  IdentifiedNode,
                                                  NodeDescription};
use grapl_observe::metric_reporter::MetricReporter;
use grapl_service::{decoder::ZstdProtoDecoder,
                    serialization::IdentifiedGraphSerializer};
use log::*;
use rusoto_dynamodb::{DynamoDb,
                      DynamoDbClient};
use rusoto_sqs::SqsClient;
use sessiondb::SessionDb;
use sqs_executor::{cache::{Cache,
                           CacheResponse,
                           Cacheable},
                   errors::{CheckedError,
                            Recoverable},
                   event_handler::{CompletedEvents,
                                   EventHandler},
                   event_retriever::S3PayloadRetriever,
                   event_status::EventStatus,
                   make_ten,
                   s3_event_emitter::S3ToSqsEventNotifier,
                   time_based_key_fn};
use tonic::transport::Channel;
use grapl_graph_descriptions::graph_mutation_service::graph_mutation_rpc_client::GraphMutationRpcClient;
use crate::node_allocator::NodeAllocator;

macro_rules! wait_on {
    ($x:expr) => {{
        $x.await
    }};
}

pub mod dynamic_sessiondb;

pub mod key_cache;
pub mod sessiondb;
pub mod sessions;
pub mod node_allocator;

#[derive(Clone)]
pub struct NodeIdentifier<D, CacheT>
    where
        D: DynamoDb + Clone + Send + Sync + 'static,
        CacheT: Cache + Clone + Send + Sync + 'static,
{
    dynamic_identifier: NodeDescriptionIdentifier<D>,
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
        dynamic_identifier: NodeDescriptionIdentifier<D>,
        node_id_db: D,
        should_default: bool,
        cache: CacheT,
    ) -> Self {
        Self {
            dynamic_identifier,
            node_id_db,
            should_default,
            cache,
        }
    }

    async fn attribute_node_key(&self, node: &NodeDescription) -> Result<IdentifiedNode, Error> {
        self
            .dynamic_identifier
            .attribute_dynamic_node(&node)
            .await
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
    type InputEvent = GraphDescription;
    type OutputEvent = IdentifiedGraph;
    type Error = NodeIdentifierError;

    async fn handle_event(
        &mut self,
        unid_subgraph: GraphDescription,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        let mut attribution_failure = None;

        info!("Handling raw event");

        if unid_subgraph.is_empty() {
            warn!("Received empty subgraph");
            return Ok(IdentifiedGraph::new());
        }

        let mut dead_node_ids = HashSet::new();
        let mut unid_id_map = HashMap::new();

        let output_subgraph = unid_subgraph;
        // new method
        let mut identified_graph = IdentifiedGraph::new();
        for (old_node_key, old_node) in output_subgraph.nodes.iter() {
            let node = old_node.clone();

            let node = match self.attribute_node_key(&node).await {
                Ok(node) => node,
                Err(e) => {
                    warn!("Failed to attribute node_key with: {}", e);
                    dead_node_ids.insert(old_node_key.clone());

                    attribution_failure = Some(e);
                    continue;
                }
            };
            unid_id_map.insert(old_node_key.to_owned(), node.uid);
            identified_graph.add_node(node);
        }

        info!(
            "PRE: output_subgraph.edges.len() {}",
            output_subgraph.edges.len()
        );

        for (old_key, edge_list) in output_subgraph.edges.iter() {
            if dead_node_ids.contains(old_key) {
                continue;
            };

            for edge in &edge_list.edges {
                let from_uid = unid_id_map.get(&edge.from_node_key);
                let to_uid = unid_id_map.get(&edge.to_node_key);

                let (from_uid, to_uid) = match (from_uid, to_uid) {
                    (Some(from_uid), Some(to_uid)) => (from_uid, to_uid),
                    (Some(from_uid), None) => {
                        tracing::warn!(
                            message="Could not get node_key mapping for from_uid",
                            from_uid=?from_uid,
                        );
                        continue;
                    }
                    (None, Some(to_uid)) => {
                        tracing::warn!(
                            message="Could not get node_key mapping for to_uid",
                            to_uid=?to_uid,
                        );
                        continue;
                    }
                    (None, None) => {
                        tracing::warn!(
                            message="Could not get node_key mapping for from_uid and to_uid",
                            from_uid=?from_uid,
                            to_uid=?to_uid,
                        );
                        continue;
                    }
                };

                identified_graph.add_edge(
                    edge.edge_name.to_owned(),
                    *from_uid,
                    *to_uid,
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
            // todo: Use a better error
            if let Some(_e) = attribution_failure {
                return Err(Err(NodeIdentifierError::Unexpected));
            }
            return Ok(IdentifiedGraph::new());
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

pub async fn handler(_should_default: bool) -> Result<(), Box<dyn std::error::Error>> {
    let should_default = true;
    let (env, _guard) = grapl_config::init_grapl_env!();
    let source_queue_url = grapl_config::source_queue_url();

    tracing::info!(
        source_queue_url=?source_queue_url,
        env=?env,
        "handler_init"
    );
    let sqs_client = SqsClient::from_env();

    let cache = &mut event_caches(&env).await;

    let serializer = &mut make_ten(async { IdentifiedGraphSerializer::default() }).await;

    let s3_emitter =
        &mut s3_event_emitters_from_env(&env, time_based_key_fn, S3ToSqsEventNotifier::from(&env))
            .await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| grapl_config::env_helpers::init_s3_client(&region_str),
            ZstdProtoDecoder::default(),
            MetricReporter::new(&env.service_name),
        )
    })
        .await;

    let mutation_endpoint = grapl_config::mutation_endpoint();

    let mutation_client: GraphMutationRpcClient<Channel> =
        GraphMutationRpcClient::connect(mutation_endpoint)
            .await
            .expect("Failed to connect to graph-mutation-service");
    let node_allocator = NodeAllocator { mutation_client };
    let dynamo = DynamoDbClient::from_env();
    let dyn_session_db = SessionDb::new(dynamo.clone(), node_allocator.clone(), grapl_config::dynamic_session_table_name());
    let dyn_mapping_db = DynamicMappingDb::new(DynamoDbClient::from_env(), node_allocator);

    let dyn_node_identifier =
        NodeDescriptionIdentifier::new(dyn_session_db, dyn_mapping_db, should_default);

    let node_identifier = &mut make_ten(async {
        NodeIdentifier::new(
            dyn_node_identifier,
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
