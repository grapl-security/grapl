#![cfg(feature = "integration_tests")]
use std::sync::Arc;

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
                QueryGraphWithUidRequest,
                StringCmp,
            },
            uid_allocator::v1beta1::{
                client::UidAllocatorServiceClient,
                messages::CreateTenantKeyspaceRequest,
            },
        },
        common::v1beta1::types::NodeType,
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
async fn test_query_single_node() -> Result<(), DynError> {
    let _span = tracing::info_span!(
        "tenant_id", tenant_id=?tracing::field::Empty,
    );
    tracing::info!("starting test_query_single_node");

    let query_client_config = GraphQueryClientConfig::parse();
    let mut graph_query_client = build_grpc_client(query_client_config).await?;
    tracing::info!("connected to graph query service");

    let mutation_client_config = GraphMutationClientConfig::parse();
    let mut graph_mutation_client = build_grpc_client(mutation_client_config).await?;
    tracing::info!("connected to graph mutation service");

    // Only used to provision the keyspace. It's okay here to use the
    // otherwise-unrecommended non-cachine UidAllocator client
    let uid_allocator_client = build_grpc_client(UidAllocatorClientConfig::parse()).await?;

    let tenant_id = uuid::Uuid::new_v4();
    _span.record("tenant_id", &format!("{tenant_id}"));

    let session = get_scylla_client().await?;
    let _keyspace_name =
        provision_keyspace(session.as_ref(), tenant_id, uid_allocator_client).await?;

    let node_type = NodeType::try_from("Process").unwrap();

    let mutation::CreateNodeResponse { uid } = graph_mutation_client
        .create_node(mutation::CreateNodeRequest {
            tenant_id,
            node_type: node_type.clone(),
        })
        .await?;

    graph_mutation_client
        .set_node_property(mutation::SetNodePropertyRequest {
            tenant_id,
            uid,
            node_type: node_type.clone(),
            property_name: "process_name".try_into()?,
            property: NodeProperty {
                property: Property::ImmutableStrProp(ImmutableStrProp {
                    prop: "chrome.exe".into(),
                }),
            },
        })
        .await?;

    let graph_query = NodeQuery::root(node_type.clone())
        .with_string_comparisons(
            "process_name".try_into()?,
            vec![StringCmp::Eq("chrome.exe".to_owned(), false)],
        )
        .build();

    let response = graph_query_client
        .query_graph_with_uid(QueryGraphWithUidRequest {
            tenant_id: tenant_id.into(),
            node_uid: uid,
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

    let (returned_uid, returned_node) = matched_graph.nodes.into_iter().next().unwrap();
    assert_eq!(returned_uid, uid);
    assert_eq!(returned_uid, root_uid);

    assert_eq!(returned_node.node_type, node_type);
    assert_eq!(returned_node.string_properties.prop_map.len(), 1);

    let (returned_property_name, returned_property) = returned_node
        .string_properties
        .prop_map
        .into_iter()
        .next()
        .unwrap();

    assert_eq!(returned_property_name, "process_name".try_into()?);
    assert_eq!(&returned_property, "chrome.exe");
    drop(_span);
    Ok(())
}
