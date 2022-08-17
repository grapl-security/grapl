#![cfg(feature = "integration_tests")]
use std::sync::Arc;

use clap::Parser;
use graph_query_service::{
    config::GraphDbConfig,
    node_query::NodeQuery,
};
use rust_proto::graplinc::grapl::{
    api::graph_query_service::v1beta1::{
        client::GraphQueryClient,
        messages::{
            MatchedGraphWithUid,
            MaybeMatchWithUid,
            QueryGraphWithUidRequest,
            StringCmp,
        },
    },
    common::v1beta1::types::{
        NodeType,
        Uid,
    },
};
use scylla::{
    CachingSession,
};
use secrecy::ExposeSecret;

type DynError = Box<dyn std::error::Error + Send + Sync>;

// NOTE: temporary code to set up the keyspace before we have a service
// to set it up for us
#[tracing::instrument(skip(session), err)]
async fn provision_keyspace(
    session: &CachingSession,
    tenant_id: uuid::Uuid,
) -> Result<String, DynError> {
    let tenant_ks = format!("tenant_keyspace_{}", tenant_id.simple());

    session
        .session
        .query(
        format!(
            r"CREATE KEYSPACE IF NOT EXISTS {tenant_ks} WITH REPLICATION = {{'class' : 'SimpleStrategy', 'replication_factor' : 1}};"
        ),
        &[],
    ).await?;

    let property_table_names = [("immutable_strings", "text")];

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

#[tracing::instrument(skip(session), fields(
    keyspace = %keyspace.as_ref(),
    uid=%uid.as_i64(),
    node_type=node_type.value.as_str(),
) err)]
async fn create_node(
    session: &CachingSession,
    keyspace: impl AsRef<str>,
    uid: Uid,
    node_type: &NodeType,
) -> Result<(), DynError> {
    let keyspace = keyspace.as_ref();
    let insert = format!(
        r"INSERT INTO {keyspace}.node_type
       (uid, node_type)
       VALUES (?, ?)"
    );

    session
        .execute(insert, &(uid.as_i64(), &node_type.value))
        .await?;

    Ok(())
}

async fn insert_string(
    session: &CachingSession,
    keyspace: impl AsRef<str>,
    uid: Uid,
    populated_field: impl AsRef<str>,
    value: impl AsRef<str>,
) -> Result<(), DynError> {
    let keyspace = keyspace.as_ref();
    let insert = format!(
        r"INSERT INTO {keyspace}.immutable_strings
        (uid, populated_field, value)
        VALUES (?, ?, ?)"
    );

    session
        .execute(
            insert,
            &(uid.as_i64(), populated_field.as_ref(), value.as_ref()),
        )
        .await?;

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_query_single_node() -> Result<(), DynError> {
    let _span = tracing::info_span!(
        "tenant_id", tenant_id=?tracing::field::Empty,
    );
    tracing::info!("starting test_query_single_node");
    let graph_query_service_endpoint = std::env::var("GRAPH_QUERY_SERVICE_ENDPOINT_ADDRESS")
        .expect("GRAPH_QUERY_SERVICE_ENDPOINT_ADDRESS");
    tracing::info!(
        graph_query_service_endpoint=%graph_query_service_endpoint,
        message="connecting to graph query service"
    );
    let mut graph_query_client = GraphQueryClient::connect(graph_query_service_endpoint).await?;
    tracing::info!("connected to graph query service");
    let tenant_id = uuid::Uuid::new_v4();
    _span.record("tenant_id", &format!("{tenant_id}"));

    let session = get_scylla_client().await?;

    let keyspace_name = provision_keyspace(session.as_ref(), tenant_id).await?;

    let uid = Uid::from_u64(1).unwrap();
    let node_type = NodeType::try_from("Process").unwrap();

    create_node(session.as_ref(), &keyspace_name, uid, &node_type).await?;
    insert_string(
        session.as_ref(),
        &keyspace_name,
        uid,
        "process_name",
        "chrome.exe",
    )
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
