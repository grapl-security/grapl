#![allow(warnings)]

#[cfg(feature = "integration")]
pub mod test {
    use graph_mutation_service_lib::{mutations, upsert_manager::UpsertManager};
    use mutations::node_mutation::{NodeUpsertGenerator};
    use grapl_graph_descriptions::IdentifiedNode;
    use grapl_graph_descriptions::ImmutableUintProp;

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

    #[tokio::test]
    async fn test_upsert_incr_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let mut upsert_manager = UpsertManager {
            dgraph_client: std::sync::Arc::new(DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client.")),
            node_upsert_generator: NodeUpsertGenerator::default(),
        };

        let mut properties = HashMap::new();
        properties.insert(
            "example_id".to_string(),
            ImmutableUintProp {
                prop: 1000,
            }
                .into(),
        );
        let n0 = IdentifiedNode {
            node_key: "test_upsert_incr_only_uint-example-node-key".to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0);

        Ok(())
    }
}
