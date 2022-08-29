#![cfg(feature = "integration_tests")]
use std::sync::Arc;

use bytes::Bytes;
use clap::Parser;
use graph_query::{
    config::GraphDbConfig,
    node_query::NodeQuery,
    table_names::{
        tenant_keyspace_name,
        IMM_STRING_TABLE_NAME,
    },
};
use rust_proto::{
    client_factory::{
        build_grpc_client,
        services::{
            GraphMutationClientConfig,
            GraphQueryClientConfig,
            GraphSchemaManagerClientConfig,
            UidAllocatorClientConfig,
        },
    },
    graplinc::grapl::{
        api::{
            graph::v1beta1::{
                ImmutableStrProp,
                NodeProperty,
                Property,
            },
            graph_mutation::v1beta1::messages as mutation,
            graph_query_service::v1beta1::messages::{
                MatchedGraphWithUid,
                MaybeMatchWithUid,
                NodePropertyQuery,
                QueryGraphFromUidRequest,
                QueryGraphWithUidRequest,
                StringCmp,
            },
            graph_schema_manager::v1beta1::messages as graph_schema_manager_api,
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
};
use scylla::CachingSession;
use secrecy::ExposeSecret;

type DynError = Box<dyn std::error::Error + Send + Sync>;

// NOTE: temporary code to set up the keyspace before we have a service
// to set it up for us
#[tracing::instrument(skip(session, uid_allocator_client), err)]
async fn provision_keyspace(
    session: &CachingSession,
    tenant_id: uuid::Uuid,
    mut uid_allocator_client: UidAllocatorServiceClient,
) -> Result<String, DynError> {
    let tenant_ks = tenant_keyspace_name(tenant_id);

    session
        .session
        .query(
        format!(
            r"CREATE KEYSPACE IF NOT EXISTS {tenant_ks} WITH REPLICATION = {{'class' : 'SimpleStrategy', 'replication_factor' : 1}};"
        ),
        &[],
    ).await?;

    let property_table_names = [(IMM_STRING_TABLE_NAME, "text")];

    for (table_name, value_type) in property_table_names.into_iter() {
        session
            .session
            .query(
                format!(
                    r"CREATE TABLE IF NOT EXISTS {tenant_ks}.{table_name} (
                        uid bigint,
                        populated_field text,
                        value {value_type},
                        PRIMARY KEY (uid, populated_field)
                    )"
                ),
                &(),
            )
            .await?;
    }

    session
        .session
        .query(
            format!(
                r"CREATE TABLE IF NOT EXISTS {tenant_ks}.node_type (
                    uid bigint,
                    node_type text,
                    PRIMARY KEY (uid, node_type)
                )"
            ),
            &(),
        )
        .await?;

    session
        .session
        .query(
            format!(
                r"CREATE TABLE IF NOT EXISTS {tenant_ks}.edges (
                    source_uid bigint,
                    destination_uid bigint,
                    f_edge_name text,
                    r_edge_name text,
                    PRIMARY KEY ((source_uid, f_edge_name), destination_uid)
                )"
            ),
            &(),
        )
        .await?;

    session.session.await_schema_agreement().await?;

    uid_allocator_client
        .create_tenant_keyspace(CreateTenantKeyspaceRequest { tenant_id })
        .await?;

    Ok(tenant_ks)
}

async fn provision_example_graph_schema(tenant_id: uuid::Uuid) -> Result<(), DynError> {
    let graph_schema_manager_client_config = GraphSchemaManagerClientConfig::parse();
    let mut graph_schema_manager_client =
        build_grpc_client(graph_schema_manager_client_config).await?;

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

async fn get_scylla_client() -> Result<Arc<CachingSession>, DynError> {
    let graph_db_config = GraphDbConfig::parse();

    let mut scylla_config = scylla::SessionConfig::new();
    scylla_config.add_known_nodes_addr(&graph_db_config.graph_db_addresses[..]);
    scylla_config.auth_username = Some(graph_db_config.graph_db_auth_username.to_owned());
    scylla_config.auth_password = Some(
        graph_db_config
            .graph_db_auth_password
            .expose_secret()
            .to_owned(),
    );

    let scylla_client = Arc::new(CachingSession::from(
        scylla::Session::connect(scylla_config).await?,
        10_000,
    ));
    Ok(scylla_client)
}

#[test_log::test(tokio::test)]
async fn test_query_two_attached_nodes() -> Result<(), DynError> {
    let _span = tracing::info_span!(
        "tenant_id", tenant_id=?tracing::field::Empty,
    );

    let query_client_config = GraphQueryClientConfig::parse();
    let mut graph_query_client = build_grpc_client(query_client_config).await?;

    let mutation_client_config = GraphMutationClientConfig::parse();
    let mut graph_mutation_client = build_grpc_client(mutation_client_config).await?;

    // Only used to provision the keyspace. It's okay here to use the
    // otherwise-unrecommended non-caching UidAllocator client.
    let uid_allocator_client = build_grpc_client(UidAllocatorClientConfig::parse()).await?;

    let tenant_id = uuid::Uuid::new_v4();
    _span.record("tenant_id", &format!("{tenant_id}"));

    let session = get_scylla_client().await?;
    let _keyspace_name =
        provision_keyspace(session.as_ref(), tenant_id, uid_allocator_client).await?;

    // Provision a Schema so we can test how edges work
    provision_example_graph_schema(tenant_id).await?;

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

// TODO: test `with_edge_to`
