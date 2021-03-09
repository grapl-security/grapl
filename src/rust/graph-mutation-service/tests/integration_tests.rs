#[cfg(feature = "integration")]
pub mod test {
    use std::collections::HashMap;

    use dgraph_tonic::{Client as DgraphClient,
                       Query};
    use graph_mutation_service_lib::{mutations,
                                     upsert_manager::UpsertManager};
    use grapl_graph_descriptions::{DecrementOnlyUintProp,
                                   IdentifiedEdge,
                                   IdentifiedNode,
                                   ImmutableUintProp,
                                   IncrementOnlyUintProp};
    use mutations::{edge_mutation::EdgeUpsertGenerator,
                    node_mutation::NodeUpsertGenerator};
    use tonic::transport::Channel;
    use grapl_graph_descriptions::graph_mutation_service::graph_mutation_rpc_client::GraphMutationRpcClient;
    use grapl_graph_descriptions::graph_mutation_service::CreateNodeSuccess;
    use grapl_graph_descriptions::graph_mutation_service::CreateNodeRequest;
    use grapl_graph_descriptions::graph_mutation_service::create_node_result;

    fn uid_from_str(hex_encoded: &str) -> u64 {
        u64::from_str_radix(&hex_encoded[2..], 16).expect("invalid uid")
    }

    fn init_test_env() {
        let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
            .finish();
        let _ = ::tracing::subscriber::set_global_default(subscriber);
    }

    async fn retrieve_node(dgraph_client: &DgraphClient, uid: u64) -> serde_json::Value {
        let query = format!(r#"
          {{
            q0(func: uid({})) {{
                uid,
                expand(Process) {{
                    uid,
                    expand(Process)
                }}
            }}
          }}
        "#, uid);
        let mut txn = dgraph_client.new_read_only_txn();
        let res = txn
            .query(query)
            .await
            .expect("query failed");
        let value: serde_json::Value = serde_json::from_slice(&res.json).expect("json failed");
        value["q0"][0].clone()
    }

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

    #[tokio::test]
    async fn test_upsert_immutable_only_uint_after_create() -> Result<(), Box<dyn std::error::Error>>
    {
        init_test_env();
        let node_allocator = make_allocator().await;
        let uid = create_process(node_allocator).await;

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };

        // First there's no process_id
        let properties = HashMap::new();
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert!(res["process_id"].is_null());

        // Then we set the process_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            ImmutableUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_immutable_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let node_allocator = make_allocator().await;
        let uid = create_process(node_allocator).await;
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
            ImmutableUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        // If we try to upsert a any other integer it will not be stored
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            ImmutableUintProp { prop: 900 }.into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_incr_only_uint_after_create() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let node_allocator = make_allocator().await;
        let uid = create_process(node_allocator).await;
        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };

        // First there's no process_id
        let properties = HashMap::new();
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert!(res["process_id"].is_null());

        // Then we set the process_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            IncrementOnlyUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_incr_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let node_allocator = make_allocator().await;
        let uid = create_process(node_allocator).await;
        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };


        // We set the process_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            IncrementOnlyUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        // If we try to upsert a smaller integer it will not be stored
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            IncrementOnlyUintProp { prop: 900 }.into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));

        // If we try to upsert a larger integer it will be stored
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            IncrementOnlyUintProp { prop: 1100 }.into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert_eq!(1100, res["process_id"].as_u64().expect("process_id"));

        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_decr_only_uint_after_create() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let node_allocator = make_allocator().await;
        let uid = create_process(node_allocator).await;
        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };

        // First there's no process_id
        let properties = HashMap::new();
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert!(res["process_id"].is_null());

        // Then we set the process_id to 1000
        let mut properties = HashMap::new();
        properties.insert(
            "process_id".to_string(),
            DecrementOnlyUintProp { prop: 1000 }.into(),
        );
        let n0 = IdentifiedNode {
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid).await;
        assert_eq!(1000, res["process_id"].as_u64().expect("process_id"));
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_decr_only_uint() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        let node_allocator = make_allocator().await;
        let uid = create_process(node_allocator).await;

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
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(
            dgraph_client.as_ref(),
            uid,
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
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(
            dgraph_client.as_ref(),
            uid,
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
            uid,
            node_type: "Process".to_string(),
            properties,
        };

        upsert_manager.upsert_node(&n0).await?;
        let res = retrieve_node(
            dgraph_client.as_ref(),
            uid,
        )
            .await;
        assert_eq!(900, res["process_id"].as_u64().expect("process_id"));

        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_edge() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let node_allocator = make_allocator().await;
        let uid_0 = create_process(node_allocator.clone()).await;
        let uid_1 = create_process(node_allocator).await;

        let mg_alphas = grapl_config::mg_alphas();
        let dgraph_client = std::sync::Arc::new(
            DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."),
        );
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };

        let n0 = IdentifiedNode {
            uid: uid_0,
            node_type: "Process".to_string(),
            properties: HashMap::new(),
        };

        let n1 = IdentifiedNode {
            uid: uid_1,
            node_type: "Process".to_string(),
            properties: HashMap::new(),
        };
        upsert_manager.upsert_node(&n0).await?;
        upsert_manager.upsert_node(&n1).await?;

        let forward_edge = IdentifiedEdge {
            from_uid: uid_0,
            to_uid: uid_1,
            edge_name: "children".to_string(),
        };
        let reverse_edge = IdentifiedEdge {
            from_uid: uid_1,
            to_uid: uid_0,
            edge_name: "parent".to_string(),
        };
        upsert_manager
            .upsert_edge(forward_edge.clone(), reverse_edge.clone())
            .await?;
        let res = retrieve_node(dgraph_client.as_ref(), uid_0).await;
        assert_eq!(
            uid_1,
            uid_from_str(res["children"][0]["uid"].as_str().expect("as_str")),
            "{}",
            res
        );

        let res = retrieve_node(dgraph_client.as_ref(), uid_1).await;
        assert_eq!(
            uid_0,
            uid_from_str(res["parent"]["uid"].as_str().expect("as_str")),
            "{}",
            res
        );

        Ok(())
    }
}
