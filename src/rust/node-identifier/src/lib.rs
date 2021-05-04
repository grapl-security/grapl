// #![allow(unused_must_use)]

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
use grapl_service::{decoder::ProtoDecoder,
                    serialization::IdentifiedGraphSerializer};
use log::*;
use rusoto_dynamodb::{DynamoDb,
                      DynamoDbClient};
use rusoto_sqs::SqsClient;
use sessiondb::SessionDb;
use sqs_executor::{cache::{Cache,
                           Cacheable},
                   errors::{CheckedError,
                            Recoverable},
                   event_handler::{CompletedEvents,
                                   EventHandler},
                   make_ten,
                   s3_event_emitter::S3ToSqsEventNotifier,
                   s3_event_retriever::S3PayloadRetriever,
                   time_based_key_fn};

macro_rules! wait_on {
    ($x:expr) => {{
        $x.await
    }};
}

pub mod dynamic_sessiondb;

pub mod sessiondb;
pub mod sessions;

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

    // todo: We should be yielding IdentifiedNode's here
    async fn attribute_node_key(&self, node: &NodeDescription) -> Result<IdentifiedNode, Error> {
        let new_node = self
            .dynamic_identifier
            .attribute_dynamic_node(&node)
            .await?;
        Ok(new_node.into())
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
    // todo: IdentifiedGraph's should be emitted
    type OutputEvent = IdentifiedGraph;
    type Error = NodeIdentifierError;

    async fn handle_event(
        &mut self,
        unid_subgraph: GraphDescription,
        _completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        let mut attribution_failure = None;

        info!("Handling raw event");

        if unid_subgraph.is_empty() {
            warn!("Received empty subgraph");
            return Ok(IdentifiedGraph::new());
        }

        info!(
            "unid_subgraph: {} nodes {} edges",
            unid_subgraph.nodes.len(),
            unid_subgraph.edges.len(),
        );

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
                    dead_node_ids.insert(node.clone_node_key());

                    attribution_failure = Some(e);
                    continue;
                }
            };
            unid_id_map.insert(old_node_key.to_owned(), node.clone_node_key());
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
                let from_key = unid_id_map.get(&edge.from_node_key);
                let to_key = unid_id_map.get(&edge.to_node_key);

                let (from_key, to_key) = match (from_key, to_key) {
                    (Some(from_key), Some(to_key)) => (from_key, to_key),
                    (Some(from_key), None) => {
                        tracing::warn!(
                            message="Could not get node_key mapping for from_key",
                            from_key=?from_key,
                        );
                        continue;
                    }
                    (None, Some(to_key)) => {
                        tracing::warn!(
                            message="Could not get node_key mapping for to_key",
                            to_key=?to_key,
                        );
                        continue;
                    }
                    (None, None) => {
                        tracing::warn!(
                            message="Could not get node_key mapping for from_key and to_key",
                            from_key=?from_key,
                            to_key=?to_key,
                        );
                        continue;
                    }
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
            // todo: Use a better error
            if let Some(_e) = attribution_failure {
                return Err(Err(NodeIdentifierError::Unexpected));
            }
            return Ok(IdentifiedGraph::new());
        }

        if !dead_node_ids.is_empty() || attribution_failure.is_some() {
            info!(
                "Partial Success, identified {} nodes",
                identified_graph.nodes.len()
            );
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
            ProtoDecoder::default(),
            MetricReporter::new(&env.service_name),
        )
    })
    .await;

    let dynamo = DynamoDbClient::from_env();
    let dyn_session_db = SessionDb::new(dynamo.clone(), grapl_config::dynamic_session_table_name());
    let dyn_mapping_db = DynamicMappingDb::new(DynamoDbClient::from_env());

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

    async fn all_exist<CA>(&mut self, cacheables: &[CA]) -> bool
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        let self_cache = self.cache.lock().unwrap();

        cacheables
            .into_iter()
            .all(|c| self_cache.contains(&c.identity()))
    }

    async fn store<CA>(&mut self, cacheable: CA) -> Result<(), HashCacheError>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        let mut self_cache = self.cache.lock().unwrap();
        self_cache.insert(cacheable.identity());
        Ok(())
    }

    async fn filter_cached<CA>(&mut self, cacheables: &[CA]) -> Vec<CA>
    where
        CA: Cacheable + Send + Sync + Clone + 'static,
    {
        let self_cache = self.cache.lock().unwrap();

        cacheables
            .iter()
            .cloned()
            .filter(|c| self_cache.contains(&c.identity()))
            .collect()
    }
}
