#![allow(warnings)]

#[cfg(feature = "integration")]
pub mod test {
    use graph_mutation_service_lib::{mutations, upsert_manager::UpsertManager};
    use mutations::node_mutation::{NodeUpsertGenerator};
    use grapl_graph_descriptions::IdentifiedNode;
    use grapl_graph_descriptions::ImmutableUintProp;
    use grapl_graph_descriptions::IncrementOnlyUintProp;
    use grapl_graph_descriptions::DecrementOnlyUintProp;

    use std::{collections::HashMap,
              sync::{Arc,
                     Once}};

    use dgraph_query_lib::{predicate::{Field,
                                       Predicate},
                           schema::{Indexing,
                                    PredicateDefinition,
                                    PredicateType,
                                    Schema,
                                    SchemaDefinition}
    };
    use dgraph_tonic::{Client as DgraphClient,
                       Query};

    fn init_test_env() {
        let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
            .finish();
        let _ = ::tracing::subscriber::set_global_default(subscriber);

        static START: Once = Once::new();
        START.call_once(|| {
            let schema = Schema::new()
                .add_definition(
                    SchemaDefinition::new("ExampleNode")
                        .add_predicate(
                            PredicateDefinition::new("example_id", PredicateType::INT)
                                .add_index(Indexing::INT),
                        )
                        .add_predicate(
                            PredicateDefinition::new("node_key", PredicateType::String)
                                .add_index(Indexing::EXACT)
                                .upsert(),
                        )
                        .add_predicate(
                            PredicateDefinition::new("example_name", PredicateType::String)
                                .add_index(Indexing::TRIGRAM),
                        )
                        .add_predicate(PredicateDefinition::new(
                            "to_many_edge",
                            PredicateType::UIDArray,
                        ))
                        .add_predicate(PredicateDefinition::new(
                            "to_single_edge",
                            PredicateType::UID,
                        )),
                )
                .to_string();

            std::thread::spawn(move || {
                let mut rt = tokio::runtime::Runtime::new().expect("failed to init runtime");
                rt.block_on(async {
                    let dgraph_client = DgraphClient::new("http://127.0.0.1:9080")
                        .expect("Failed to create dgraph client.");

                    dgraph_client
                        .alter(dgraph_tonic::Operation {
                            drop_all: true,
                            ..Default::default()
                        })
                        .await
                        .expect("alter failed");

                    dgraph_client
                        .alter(dgraph_tonic::Operation {
                            schema,
                            ..Default::default()
                        })
                        .await
                        .expect("alter failed");
                });
            })
                .join()
                .expect("provision failed");
        });
    }

    async fn retrieve_node(dgraph_client: &DgraphClient, node_key: &str) -> serde_json::Value {
        let query = r#"
          query q0($a: string) {
            q0(func: eq(node_key, $a)) {
                uid,
                expand(ExampleNode)
            }
          }
        "#;
        let mut variables = HashMap::with_capacity(1);
        variables.insert("$a".to_string(), node_key.to_string());
        let mut txn = dgraph_client.new_read_only_txn();
        let res = txn.query_with_vars(query, variables).await.expect("query failed");
        let value: serde_json::Value = serde_json::from_slice(&res.json).expect("json failed");
        value["q0"][0].clone()
    }

    #[tokio::test]
    async fn test_upsert_immutable_only_uint_after_create() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let dgraph_client = std::sync::Arc::new(DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client."));
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
        };
        let node_key = "test_upsert_immutable_only_uint_after_create-example-node-key";
        // First there's no example_id
        let mut properties = HashMap::new();
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert!(res["example_id"].is_null());

        // Then we set the example_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            ImmutableUintProp {
                prop: 1000,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["example_id"].as_u64().expect("example_id"));
        Ok(())
    }


    #[tokio::test]
    async fn test_upsert_immutable_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let dgraph_client = std::sync::Arc::new(DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client."));
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
        };

        let node_key = "test_upsert_immutable_only_uint-example-node-key";

        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            ImmutableUintProp {
                prop: 1000,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["example_id"].as_u64().expect("example_id"));

        // If we try to upsert a any other integer it will not be stored
        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            ImmutableUintProp {
                prop: 900,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["example_id"].as_u64().expect("example_id"));

        Ok(())
    }


    #[tokio::test]
    async fn test_upsert_incr_only_uint_after_create() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let dgraph_client = std::sync::Arc::new(DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client."));
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
        };
        let node_key = "test_upsert_incr_only_uint_after_create-example-node-key";
        // First there's no example_id
        let mut properties = HashMap::new();
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert!(res["example_id"].is_null());

        // Then we set the example_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            IncrementOnlyUintProp {
                prop: 1000,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["example_id"].as_u64().expect("example_id"));
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_incr_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let dgraph_client = std::sync::Arc::new(DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client."));
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
        };
        let node_key = "test_upsert_incr_only_uint-example-node-key";

        // We set the example_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            IncrementOnlyUintProp {
                prop: 1000,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["example_id"].as_u64().expect("example_id"));

        // If we try to upsert a smaller integer it will not be stored
        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            IncrementOnlyUintProp {
                prop: 900,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["example_id"].as_u64().expect("example_id"));


        // If we try to upsert a larger integer it will be stored
        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            IncrementOnlyUintProp {
                prop: 1100,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1100, res["example_id"].as_u64().expect("example_id"));

        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_decr_only_uint_after_create() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let dgraph_client = std::sync::Arc::new(DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client."));
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
        };
        let node_key = "test_upsert_decr_only_uint_after_create-example-node-key";
        // First there's no example_id
        let mut properties = HashMap::new();
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert!(res["example_id"].is_null());

        // Then we set the example_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            DecrementOnlyUintProp {
                prop: 1000,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["example_id"].as_u64().expect("example_id"));
        Ok(())
    }


    #[tokio::test]
    async fn test_upsert_decr_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let dgraph_client = std::sync::Arc::new(DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client."));
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
        };

        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            DecrementOnlyUintProp {
                prop: 1000,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: "test_upsert_decr_only_uint-example-node-key".to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), "test_upsert_decr_only_uint-example-node-key").await;
        assert_eq!(1000, res["example_id"].as_u64().expect("example_id"));

        // If we try to upsert a larger integer it will not be stored
        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            DecrementOnlyUintProp {
                prop: 1100,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: "test_upsert_decr_only_uint-example-node-key".to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), "test_upsert_decr_only_uint-example-node-key").await;
        assert_eq!(1000, res["example_id"].as_u64().expect("example_id"));


        // If we try to upsert a smaller integer it will be stored
        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            DecrementOnlyUintProp {
                prop: 900,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: "test_upsert_decr_only_uint-example-node-key".to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await;
        let res = retrieve_node(dgraph_client.as_ref(), "test_upsert_decr_only_uint-example-node-key").await;
        assert_eq!(900, res["example_id"].as_u64().expect("example_id"));

        Ok(())
    }
}
