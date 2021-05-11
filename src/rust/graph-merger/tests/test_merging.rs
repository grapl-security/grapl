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
    use std::{
        collections::HashMap,
        sync::{
            Arc,
        },
    };

    use dgraph_query_lib::{
        condition::{
            Condition,
            ConditionValue,
        },
        predicate::{
            Field,
            Predicate,
        },
        queryblock::QueryBlockType,
        EdgeBuilder,
        QueryBlockBuilder,
        QueryBuilder,
        ToQueryString,
    };
    use dgraph_tonic::{
        Client as DgraphClient,
        Query,
    };
    use graph_merger_lib::{
        service::GraphMerger,
    };
    use grapl_graph_descriptions::{
        graph_mutation_service::graph_mutation_rpc_client::GraphMutationRpcClient,
        *,
    };
    use grapl_observe::metric_reporter::MetricReporter;
    use sqs_executor::{
        cache::NopCache,
        event_handler::{
            CompletedEvents,
            EventHandler,
        },
    };

    async fn query_for_uid(dgraph_client: Arc<DgraphClient>, node_key: &str) -> u64 {
        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::EQ(
                "node_key".to_string(),
                ConditionValue::string(node_key),
            ))
            .predicates(vec![Predicate::Field(Field::new("uid"))])
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

        let m: HashMap<String, Vec<HashMap<String, String>>> =
            serde_json::from_slice(&response.json).expect("response failed to parse");
        let m = m.into_iter().next().unwrap().1;
        debug_assert!((m.len() == 1) || (m.len() == 0));

        let uid = &m[0]["uid"][2..];
        let uid = u64::from_str_radix(uid, 16).expect("uid is not valid hex");
        uid
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
        let node_key0 = "test_upsert_edge_and_retrieve-node-key0";
        let node_key1 = "test_upsert_edge_and_retrieve-node-key1";

        let mut properties = HashMap::new();
        properties.insert(
            "process_name".to_string(),
            ImmutableStrProp {
                prop: "foobar".to_string(),
            }
            .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key0.to_string(),
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
            node_key: node_key1.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        identified_graph.add_node(n0);
        identified_graph.add_node(n1);

        identified_graph.add_edge(
            "children".to_string(),
            node_key0.to_string(),
            node_key1.to_string(),
        );

        identified_graph.add_edge(
            "parent".to_string(),
            node_key0.to_string(),
            node_key1.to_string(),
        );

        let _merged_graph = graph_merger
            .handle_event(identified_graph, &mut CompletedEvents::default())
            .await
            .expect("Failed to merged graph");

        let node_uid_0 = query_for_uid(dgraph_client.clone(), node_key0).await;
        let node_uid_1 = query_for_uid(dgraph_client.clone(), node_key1).await;
        assert_ne!(node_uid_0, node_uid_1);
        assert_ne!(node_uid_0, 0);
        assert_ne!(node_uid_1, 0);

        let to_many_res = query_for_edge(dgraph_client.clone(), node_uid_0, "children").await;

        let to_single_res = query_for_edge(dgraph_client.clone(), node_uid_1, "parent").await;

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

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );

        let node_key = "test_upsert_idempotency-example-node-key";
        let mut properties = HashMap::new();
        properties.insert(
            "process_name".to_string(),
            ImmutableStrProp {
                prop: "foobar".to_string(),
            }
            .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
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

        // If we query for multiple nodes by node_key we should only ever receive one
        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::EQ(
                "node_key".to_string(),
                ConditionValue::string(node_key),
            ))
            .predicates(vec![Predicate::Field(Field::new("uid"))])
            .first(2)
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

        let m: HashMap<String, Vec<HashMap<String, String>>> =
            serde_json::from_slice(&response.json).expect("response failed to parse");
        let m = m.into_iter().next().unwrap().1;
        debug_assert_eq!(m.len(), 1);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_upsert_multifield() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
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

        let node_key = "test_upsert_multifield-example-node-key";
        let mut properties = HashMap::new();
        properties.insert(
            "process_name".to_string(),
            ImmutableStrProp {
                prop: "test_upsert_multifield".to_string(),
            }
            .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };
        let mut identified_graph = IdentifiedGraph::new();
        identified_graph.add_node(n0);

        let _merged_graph = graph_merger
            .handle_event(identified_graph, &mut CompletedEvents::default())
            .await
            .expect("Failed to merged graph");

        // If we query for multiple nodes by node_key we should only ever receive one
        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::EQ(
                "node_key".to_string(),
                ConditionValue::string(node_key),
            ))
            .predicates(vec![
                Predicate::Field(Field::new("uid")),
                Predicate::Field(Field::new("process_name")),
            ])
            // .first(2)
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

        let m: HashMap<String, Vec<HashMap<String, String>>> =
            serde_json::from_slice(&response.json).expect("response failed to parse");
        let mut m = m.into_iter().next().unwrap().1;
        debug_assert_eq!(m.len(), 1);
        let mut m = m.remove(0);
        let _uid = m.remove("uid").expect("uid");

        let process_name = m.remove("process_name").expect("process_name");
        assert!(m.is_empty());
        assert_eq!(process_name, "test_upsert_multifield");
        Ok(())
    }
}
