#[cfg(not(feature = "integration"))]
pub mod test {
    use std::convert::TryFrom;
    #[test]
    fn test_endpoint() {
        let dst = "http://graphmutation.grapl.test:5500";
        tonic::transport::Endpoint::try_from(dst).unwrap().uri();
    }
}

#[cfg(feature = "integration")]
pub mod test {
    use std::{collections::HashMap,
              sync::Arc};

    use dgraph_query_lib::{condition::{Condition},
                           predicate::{Field,
                                       Predicate},
                           queryblock::QueryBlockType,
                           EdgeBuilder,
                           QueryBlockBuilder,
                           QueryBuilder,
                           ToQueryString};
    use dgraph_tonic::{Client as DgraphClient, Query, Channel};
    use graph_merger_lib::service::GraphMerger;
    use grapl_graph_descriptions::{graph_mutation_service::graph_mutation_rpc_client::GraphMutationRpcClient,
                                   *};
    use grapl_observe::metric_reporter::MetricReporter;
    use sqs_executor::{cache::NopCache,
                       event_handler::{CompletedEvents,
                                       EventHandler}};

    use grapl_graph_descriptions::graph_mutation_service::CreateNodeSuccess;
    use grapl_graph_descriptions::graph_mutation_service::CreateNodeRequest;
    use grapl_graph_descriptions::graph_mutation_service::create_node_result;

    async fn make_allocator() -> GraphMutationRpcClient<Channel> {
        let mutation_endpoint = grapl_config::mutation_endpoint();
        GraphMutationRpcClient::connect(mutation_endpoint).await
            .expect("Failed to connect to graph-mutation-service")
    }

    async fn create_process(mut mutation_client: GraphMutationRpcClient<Channel>) -> u64 {
        let res = mutation_client
            .create_node(CreateNodeRequest {
                node_type: "Process".to_string(),
            })
            .await
            .expect("Failed to create node");
        match res
            .into_inner()
            .rpc_result
            .unwrap() {
            create_node_result::RpcResult::Created(CreateNodeSuccess { uid }) => uid,
        }
    }

    async fn query_for_edge(
        dgraph_client: Arc<DgraphClient>,
        from_uid: u64,
        edge_name: &str,
    ) -> serde_json::Value {
        let edge = Predicate::Edge(
            EdgeBuilder::default()
                .name(edge_name.to_string())
                .predicates(vec![
                    Predicate::Field(Field::new("uid")),
                    Predicate::Field(Field::new("dgraph.type").alias("dgraph_type")),
                ])
                .build()
                .unwrap(),
        );

        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::uid(&from_uid.to_string()))
            .predicates(vec![Predicate::Field(Field::new("uid")), edge])
            .first(1)
            .build()
            .unwrap();

        let query = QueryBuilder::default()
            .query_blocks(vec![query_block])
            .build()
            .unwrap();

        let mut txn = dgraph_client.new_read_only_txn();
        let response = txn
            .query(query.to_query_string())
            .await
            .expect("query failed");

        serde_json::from_slice(&response.json).expect("response failed to parse")
    }

    fn init_test_env() {
        let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
            .finish();
        let _ = ::tracing::subscriber::set_global_default(subscriber);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_upsert_edge_and_retrieve() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let node_allocator = make_allocator().await;

        let mutation_endpoint = grapl_config::mutation_endpoint();
        tracing::debug!(message="Connecting to GraphMutationService", endpoint=?mutation_endpoint);

        let mut graph_merger = GraphMerger::new(
            GraphMutationRpcClient::connect(mutation_endpoint.clone())
                .await
                .unwrap_or_else(|e| {
                    panic!(
                        "Failed to connect to graph-mutation-service {} {:?}",
                        mutation_endpoint, e
                    )
                }),
            MetricReporter::new("test_upsert_edge_and_retrieve"),
            NopCache {},
        )
        .await;

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );

        let mut identified_graph = IdentifiedGraph::new();

        let uid0 = create_process(node_allocator.clone()).await;
        let uid1 = create_process(node_allocator).await;

        let mut properties = HashMap::new();
        properties.insert(
            "process_name".to_string(),
            ImmutableStrProp {
                prop: "foobar".to_string(),
            }
            .into(),
        );
        let n0 = IdentifiedNode {
            uid: uid0,
            node_type: "Process".to_string(),
            properties,
        };

        let mut properties = HashMap::new();
        properties.insert(
            "process_name".to_string(),
            ImmutableStrProp {
                prop: "baz".to_string(),
            }
            .into(),
        );

        let n1 = IdentifiedNode {
            uid: uid1,
            node_type: "Process".to_string(),
            properties,
        };

        identified_graph.add_node(n0);
        identified_graph.add_node(n1);

        identified_graph.add_edge(
            "children".to_string(),
            uid0,
            uid1,
        );

        identified_graph.add_edge(
            "parent".to_string(),
            uid0,
            uid1,
        );

        let _merged_graph = graph_merger
            .handle_event(identified_graph, &mut CompletedEvents::default())
            .await
            .expect("Failed to merged graph");

        let to_many_res = query_for_edge(dgraph_client.clone(), uid0, "children").await;

        let to_single_res = query_for_edge(dgraph_client.clone(), uid1, "parent").await;

        let to_many_res = to_many_res
            .as_object()
            .expect("children.as_object")
            .values()
            .next()
            .expect("children empty array");
        let to_single_res = to_single_res
            .as_object()
            .expect("parent.as_object")
            .values()
            .next()
            .expect("parent empty array");

        let tm_from = to_many_res[0]["uid"].as_str().expect("tm_from");
        let tm_to = to_many_res[0]["children"][0]["uid"]
            .as_str()
            .expect("tm_to");

        let ts_from = to_single_res[0]["uid"].as_str().expect("ts_from");
        let ts_to = to_single_res[0]["parent"]["uid"].as_str().expect("ts_to");

        assert_eq!(tm_from, ts_to);
        assert_eq!(tm_to, ts_from);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_upsert_idempotency() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let node_allocator = make_allocator().await;
        let uid = create_process(node_allocator).await;
        let mutation_endpoint = grapl_config::mutation_endpoint();
        tracing::debug!(message="Connecting to GraphMutationService", endpoint=?mutation_endpoint);

        let graph_merger = GraphMerger::new(
            GraphMutationRpcClient::connect(mutation_endpoint.clone())
                .await
                .unwrap_or_else(|e| {
                    panic!(
                        "Failed to connect to graph-mutation-service {} {:?}",
                        mutation_endpoint, e
                    )
                }),
            MetricReporter::new("test_upsert_edge_and_retrieve"),
            NopCache {},
        )
        .await;

        let mut properties = HashMap::new();
        properties.insert(
            "process_name".to_string(),
            ImmutableStrProp {
                prop: "foobar".to_string(),
            }
            .into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        let upsert_futs: Vec<_> = (0..10)
            .map(|_| {
                let n0 = n0.clone();
                let mut graph_merger = graph_merger.clone();
                async move {
                    let mut identified_graph = IdentifiedGraph::new();
                    identified_graph.add_node(n0);

                    let merged_graph = graph_merger
                        .handle_event(identified_graph, &mut CompletedEvents::default())
                        .await
                        .expect("Failed to merged graph");
                    merged_graph
                }
            })
            .collect();

        let mut merged_graphs = Vec::with_capacity(upsert_futs.len());
        for upsert_fut in upsert_futs.into_iter() {
            merged_graphs.push(upsert_fut.await);
        }

        for merged_graph in merged_graphs {
            assert_eq!(merged_graph.nodes.len(), 1);
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_upsert_multifield() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let node_allocator = make_allocator().await;
        let uid = create_process(node_allocator).await;
        let mutation_endpoint = grapl_config::mutation_endpoint();
        tracing::debug!(message="Connecting to GraphMutationService", endpoint=?mutation_endpoint);

        let mut graph_merger = GraphMerger::new(
            GraphMutationRpcClient::connect(mutation_endpoint.clone())
                .await
                .unwrap_or_else(|e| {
                    panic!(
                        "Failed to connect to graph-mutation-service {} {:?}",
                        mutation_endpoint, e
                    )
                }),
            MetricReporter::new("test_upsert_edge_and_retrieve"),
            NopCache {},
        )
        .await;

        let mut properties = HashMap::new();
        properties.insert(
            "process_name".to_string(),
            ImmutableStrProp {
                prop: "test_upsert_multifield".to_string(),
            }
            .into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };
        let mut identified_graph = IdentifiedGraph::new();
        identified_graph.add_node(n0);

        let _merged_graph = graph_merger
            .handle_event(identified_graph, &mut CompletedEvents::default())
            .await
            .expect("Failed to merged graph");

        Ok(())
    }
}
