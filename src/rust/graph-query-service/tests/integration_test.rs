#![cfg(feature = "integration_tests")]
use bytes::Bytes;
use clap::Parser;
use graph_query::node_query::NodeQuery;
use rust_proto::{
    client_factory::services::{
        GraphMutationClientConfig,
        GraphQueryClientConfig,
        GraphSchemaManagerClientConfig,
        ScyllaProvisionerClientConfig,
        UidAllocatorClientConfig,
    },
    graplinc::grapl::{
        api::{
            graph::v1beta1::{
                ImmutableStrProp,
                IncrementOnlyUintProp,
                NodeProperty,
                Property,
            },
            graph_mutation::v1beta1::{
                client::GraphMutationClient,
                messages as mutation,
            },
            graph_query_service::v1beta1::{
                client::GraphQueryClient,
                messages::{
                    MatchedGraphWithUid,
                    MaybeMatchWithUid,
                    NodePropertyQuery,
                    QueryGraphFromUidRequest,
                    QueryGraphWithUidRequest,
                    StringCmp,
                },
            },
            graph_schema_manager::v1beta1::{
                client::GraphSchemaManagerClient,
                messages as graph_schema_manager_api,
            },
            scylla_provisioner::v1beta1::{
                client::ScyllaProvisionerClient,
                messages as scylla_provisioner_msgs,
            },
            uid_allocator::v1beta1::{
                client::UidAllocatorServiceClient,
                messages::CreateTenantKeyspaceRequest,
            },
        },
        common::v1beta1::types::{
            EdgeName,
            NodeType,
        },
    },
    protocol::service_client::ConnectWithConfig,
};

async fn provision_example_graph_schema(tenant_id: uuid::Uuid) -> eyre::Result<()> {
    let graph_schema_manager_client_config = GraphSchemaManagerClientConfig::parse();
    let mut graph_schema_manager_client =
        GraphSchemaManagerClient::connect_with_config(graph_schema_manager_client_config).await?;

    fn get_example_graphql_schema() -> Result<Bytes, std::io::Error> {
        // This path is created in rust/Dockerfile
        let path = "/test-fixtures/example_schemas/example.graphql";
        std::fs::read(path).map(Bytes::from)
    }

    graph_schema_manager_client
        .deploy_schema(graph_schema_manager_api::DeploySchemaRequest {
            tenant_id,
            schema: get_example_graphql_schema().unwrap(),
            schema_type: graph_schema_manager_api::SchemaType::GraphqlV0,
            schema_version: 0,
        })
        .await?;
    Ok(())
}

// This should probably replaced by a refactored E2eTestContext at some point.
struct GraphQueryIntegTestSetup {
    graph_query_client: GraphQueryClient,
    graph_mutation_client: GraphMutationClient,
    tenant_id: uuid::Uuid,
    _span: tracing::Span,
}

impl GraphQueryIntegTestSetup {
    pub async fn setup() -> eyre::Result<GraphQueryIntegTestSetup> {
        // <setup>
        let _span = tracing::info_span!(
            "tenant_id", tenant_id=?tracing::field::Empty,
        );

        let query_client_config = GraphQueryClientConfig::parse();
        let graph_query_client = GraphQueryClient::connect_with_config(query_client_config).await?;

        let mutation_client_config = GraphMutationClientConfig::parse();
        let graph_mutation_client =
            GraphMutationClient::connect_with_config(mutation_client_config).await?;

        let provisioner_client_config = ScyllaProvisionerClientConfig::parse();
        let mut provisioner_client =
            ScyllaProvisionerClient::connect_with_config(provisioner_client_config).await?;

        let tenant_id = uuid::Uuid::new_v4();
        _span.record("tenant_id", &format!("{tenant_id}"));

        provisioner_client
            .provision_graph_for_tenant(scylla_provisioner_msgs::ProvisionGraphForTenantRequest {
                tenant_id,
            })
            .await?;

        // Only used to provision the keyspace. It's okay here to use the
        // otherwise-unrecommended non-caching UidAllocator client.

        let mut uid_allocator_client =
            UidAllocatorServiceClient::connect_with_config(UidAllocatorClientConfig::parse())
                .await?;
        uid_allocator_client
            .create_tenant_keyspace(CreateTenantKeyspaceRequest { tenant_id })
            .await?;

        // Provision a Schema so we can test how edges work
        provision_example_graph_schema(tenant_id).await?;

        Ok(Self {
            graph_query_client,
            graph_mutation_client,
            tenant_id,
            _span,
        })
    }
}

#[test_log::test(tokio::test)]
async fn test_query_two_attached_nodes() -> eyre::Result<()> {
    let GraphQueryIntegTestSetup {
        mut graph_query_client,
        mut graph_mutation_client,
        tenant_id,
        _span,
    } = GraphQueryIntegTestSetup::setup().await?;

    let process_node_type = NodeType::try_from("Process").unwrap();
    let file_node_type = NodeType::try_from("File").unwrap();

    let mutation::CreateNodeResponse {
        uid: first_node_uid,
    } = graph_mutation_client
        .create_node(mutation::CreateNodeRequest {
            tenant_id,
            node_type: process_node_type.clone(),
        })
        .await?;

    graph_mutation_client
        .set_node_property(mutation::SetNodePropertyRequest {
            tenant_id,
            uid: first_node_uid,
            node_type: process_node_type.clone(),
            property_name: "process_name".try_into()?,
            property: NodeProperty {
                property: Property::ImmutableStrProp(ImmutableStrProp {
                    prop: "chrome.exe".into(),
                }),
            },
        })
        .await?;

    // Add another Node - the time a File
    let mutation::CreateNodeResponse {
        uid: second_node_uid,
    } = graph_mutation_client
        .create_node(mutation::CreateNodeRequest {
            tenant_id,
            node_type: file_node_type.clone(),
        })
        .await?;

    // Create an 'binary_file' Edge from Process --> File
    let forward_edge_name = EdgeName {
        value: "binary_file".to_string(), // as defined in the example Graphql schema
    };
    let reverse_edge_name = EdgeName {
        value: "executed_as_processes".to_string(),
    };
    graph_mutation_client
        .create_edge(mutation::CreateEdgeRequest {
            edge_name: forward_edge_name.clone(),
            tenant_id,
            from_uid: first_node_uid,
            to_uid: second_node_uid,
            source_node_type: process_node_type.clone(),
        })
        .await?;

    let graph_query = NodeQuery::root(process_node_type.clone())
        .with_string_comparisons(
            "process_name".try_into()?,
            vec![StringCmp::Eq("chrome.exe".to_owned(), false)],
        )
        .build();

    // Query about just the single node
    let response = graph_query_client
        .query_graph_with_uid(QueryGraphWithUidRequest {
            tenant_id: tenant_id.into(),
            node_uid: first_node_uid,
            graph_query,
        })
        .await?;

    let (matched_graph, root_uid) = match response.maybe_match {
        MaybeMatchWithUid::Matched(MatchedGraphWithUid {
            matched_graph,
            root_uid,
        }) => (matched_graph, root_uid),
        MaybeMatchWithUid::Missed(_) => panic!("Expected a match"),
    };

    assert_eq!(matched_graph.nodes.len(), 1);
    assert_eq!(matched_graph.edges.len(), 0);

    {
        // Assertions on the root node
        let (returned_uid, returned_node) = matched_graph.nodes.into_iter().next().unwrap();
        assert_eq!(returned_uid, first_node_uid);
        assert_eq!(returned_uid, root_uid);

        assert_eq!(returned_node.node_type, process_node_type);
        assert_eq!(returned_node.string_properties.prop_map.len(), 1);

        let (returned_property_name, returned_property) = returned_node
            .string_properties
            .prop_map
            .into_iter()
            .next()
            .unwrap();

        assert_eq!(returned_property_name, "process_name".try_into()?);
        assert_eq!(&returned_property, "chrome.exe");
    }

    // Now try to query the 'full' graph from that root uid
    let graph_query = NodeQuery::root(process_node_type.clone())
        .with_shared_edge(
            forward_edge_name.clone(),
            reverse_edge_name.clone(),
            NodePropertyQuery::new(file_node_type.clone()),
            |_| {},
        )
        .build();

    let response = graph_query_client
        .query_graph_from_uid(QueryGraphFromUidRequest {
            tenant_id: tenant_id.into(),
            node_uid: first_node_uid,
            graph_query,
        })
        .await?;

    let matched_graph = response.matched_graph.expect("Expected a matched graph");

    assert_eq!(matched_graph.nodes.len(), 2);
    assert_eq!(matched_graph.edges.len(), 2); // forward and reverse edge

    tracing::info!(
        nodes=?matched_graph.nodes,
        edges=?matched_graph.edges,
    );

    // Assertions on the edges
    for ((source_uid, edge_name), hash_set) in matched_graph.edges.into_iter() {
        assert_eq!(hash_set.len(), 1);
        if edge_name == forward_edge_name {
            assert_eq!(first_node_uid, source_uid);
            assert!(
                hash_set.contains(&second_node_uid),
                "expected {second_node_uid:?} in {hash_set:?}"
            );
        } else if edge_name == reverse_edge_name {
            assert_eq!(second_node_uid, source_uid);
            assert!(
                hash_set.contains(&first_node_uid),
                "expected {first_node_uid:?} in {hash_set:?}"
            );
        } else {
            panic!("Unknown edge_name {edge_name} (uid={source_uid:?})");
        }
    }

    drop(_span);
    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_set_node_property() -> eyre::Result<()> {
    let GraphQueryIntegTestSetup {
        graph_query_client: _,
        mut graph_mutation_client,
        tenant_id,
        _span,
    } = GraphQueryIntegTestSetup::setup().await?;

    let process_node_type = NodeType::try_from("Process").unwrap();

    // Make a node. Do lots of SetNodePropertys on it
    let mutation::CreateNodeResponse { uid } = graph_mutation_client
        .create_node(mutation::CreateNodeRequest {
            tenant_id,
            node_type: process_node_type.clone(),
        })
        .await?;

    graph_mutation_client
        .set_node_property(mutation::SetNodePropertyRequest {
            tenant_id,
            uid,
            node_type: process_node_type.clone(),
            property_name: "process_name".try_into()?,
            property: NodeProperty {
                property: Property::ImmutableStrProp(ImmutableStrProp {
                    prop: "chrome.exe".into(),
                }),
            },
        })
        .await?;

    graph_mutation_client
        .set_node_property(mutation::SetNodePropertyRequest {
            tenant_id,
            uid,
            node_type: process_node_type.clone(),
            property_name: "last_seen_time".try_into()?,
            property: NodeProperty {
                property: Property::IncrementOnlyUintProp(IncrementOnlyUintProp {
                    // arbitrary date - millis since July 2019
                    prop: 1563991514399,
                }),
            },
        })
        .await?;

    drop(_span);
    Ok(())
}

// TODO: test `with_edge_to`
