#[cfg(feature = "integration")]
pub mod test {
    use std::collections::HashMap;

    use dgraph_tonic::{
        Client as DgraphClient,
        Query,
    };
    use graph_mutation_service_lib::{
        mutations,
        upsert_manager::UpsertManager,
    };
    use grapl_graph_descriptions::{
        DecrementOnlyUintProp,
        Edge,
        IdentifiedNode,
        ImmutableUintProp,
        IncrementOnlyUintProp,
    };
    use mutations::{
        edge_mutation::EdgeUpsertGenerator,
        node_mutation::NodeUpsertGenerator,
    };

    fn init_test_env() {
        let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
            .finish();
        let _ = ::tracing::subscriber::set_global_default(subscriber);
    }

    async fn retrieve_node(dgraph_client: &DgraphClient, node_key: &str) -> serde_json::Value {
        let query = r#"
          query q0($a: string) {
            q0(func: eq(node_key, $a)) {
                uid,
                expand(Process) {
                    uid,
                    expand(Process)
                }
            }
          }
        "#;
        let mut variables = HashMap::with_capacity(1);
        variables.insert("$a".to_string(), node_key.to_string());
        let mut txn = dgraph_client.new_read_only_txn();
        let res = txn
            .query_with_vars(query, variables)
            .await
            .expect("query failed");
        let value: serde_json::Value = serde_json::from_slice(&res.json).expect("json failed");
        value["q0"][0].clone()
    }

    #[tokio::test]
    async fn test_upsert_immutable_only_uint_after_create() -> Result<(), Box<dyn std::error::Error>>
    {
        init_test_env();

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };
        let node_key = "test_upsert_immutable_only_uint_after_create-example-node-key";
        // First there's no process_id
        let properties = HashMap::new();
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert!(res["process_id"].is_null());

        // Then we set the process_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            ImmutableUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_immutable_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };

        let node_key = "test_upsert_immutable_only_uint-example-node-key";

        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            ImmutableUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        // If we try to upsert a any other integer it will not be stored
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            ImmutableUintProp { prop: 900 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_incr_only_uint_after_create() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };
        let node_key = "test_upsert_incr_only_uint_after_create-example-node-key";
        // First there's no process_id
        let properties = HashMap::new();
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert!(res["process_id"].is_null());

        // Then we set the process_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            IncrementOnlyUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_incr_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };
        let node_key = "test_upsert_incr_only_uint-example-node-key";

        // We set the process_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            IncrementOnlyUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        // If we try to upsert a smaller integer it will not be stored
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            IncrementOnlyUintProp { prop: 900 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        // If we try to upsert a larger integer it will be stored
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            IncrementOnlyUintProp { prop: 1100 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1100, res["process_id"].as_u64().expect("process_id"));

        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_decr_only_uint_after_create() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };
        let node_key = "test_upsert_decr_only_uint_after_create-example-node-key";
        // First there's no process_id
        let properties = HashMap::new();
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert!(res["process_id"].is_null());

        // Then we set the process_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            DecrementOnlyUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), node_key).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_decr_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };

        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            DecrementOnlyUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: "test_upsert_decr_only_uint-example-node-key".to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(
            dgraph_client.as_ref(),
            "test_upsert_decr_only_uint-example-node-key",
        )
        .await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        // If we try to upsert a larger integer it will not be stored
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            DecrementOnlyUintProp { prop: 1100 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: "test_upsert_decr_only_uint-example-node-key".to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(
            dgraph_client.as_ref(),
            "test_upsert_decr_only_uint-example-node-key",
        )
        .await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        // If we try to upsert a smaller integer it will be stored
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            DecrementOnlyUintProp { prop: 900 }.into(),
        );
        let n0 = IdentifiedNode {
            node_key: "test_upsert_decr_only_uint-example-node-key".to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(
            dgraph_client.as_ref(),
            "test_upsert_decr_only_uint-example-node-key",
        )
        .await;
        assert_eq!(900, res["process_id"].as_u64().expect("process_id"));

        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_edge() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };

        let node_key_0 = "test_upsert_edge-node-key0".to_string();
        let node_key_1 = "test_upsert_edge-node-key1".to_string();

        let n0 = IdentifiedNode {
            node_key: node_key_0.to_string(),
            node_type: "Process".to_string(),
            properties: HashMap::new(),
        };

        let n1 = IdentifiedNode {
            node_key: node_key_1.to_string(),
            node_type: "Process".to_string(),
            properties: HashMap::new(),
        };
        upsert_manager.upsert_node(&n0).await?;
        upsert_manager.upsert_node(&n1).await?;

        let forward_edge = Edge {
            from_node_key: node_key_0.to_string(),
            to_node_key: node_key_1.to_string(),
            edge_name: "children".to_string(),
        };
        let reverse_edge = Edge {
            from_node_key: node_key_1.to_string(),
            to_node_key: node_key_0.to_string(),
            edge_name: "parent".to_string(),
        };
        upsert_manager
            .upsert_edge(forward_edge.clone(), reverse_edge.clone())
            .await?;
        let res = retrieve_node(dgraph_client.as_ref(), &node_key_0).await;
        assert_eq!(
            node_key_1,
            res["children"][0]["node_key"].as_str().expect("as_str")
        );

        let res = retrieve_node(dgraph_client.as_ref(), &node_key_1).await;
        assert_eq!(
            node_key_0,
            res["parent"]["node_key"].as_str().expect("as_str")
        );

        Ok(())
    }
}
