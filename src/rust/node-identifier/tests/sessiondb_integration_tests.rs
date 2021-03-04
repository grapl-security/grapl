#![cfg(feature = "integration")]

use std::time::Duration;

use grapl_config::env_helpers::FromEnv;
use node_identifier::{sessiondb::SessionDb,
                      sessions::{Session,
                                 UnidSession}};
use quickcheck_macros::quickcheck;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{AttributeDefinition,
                      CreateTableError,
                      CreateTableInput,
                      CreateTableOutput,
                      DeleteTableInput,
                      DynamoDb,
                      DynamoDbClient,
                      KeySchemaElement,
                      ProvisionedThroughput};
use tokio::runtime::Runtime;
use std::collections::HashMap;
use hmap::hmap;
use grapl_graph_descriptions::{
    ImmutableUintProp,
};

fn init_test_env() {
    let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    let _ = ::tracing::subscriber::set_global_default(subscriber);
}

async fn try_create_table(
    dynamo: &impl DynamoDb,
    table_name: String,
) -> Result<CreateTableOutput, RusotoError<CreateTableError>> {
    dynamo
        .create_table(CreateTableInput {
            table_name,
            attribute_definitions: vec![
                AttributeDefinition {
                    attribute_name: "pseudo_key".into(),
                    attribute_type: "S".into(),
                },
                AttributeDefinition {
                    attribute_name: "create_time".into(),
                    attribute_type: "N".into(),
                },
            ],
            key_schema: vec![
                KeySchemaElement {
                    attribute_name: "pseudo_key".into(),
                    key_type: "HASH".into(),
                },
                KeySchemaElement {
                    attribute_name: "create_time".into(),
                    key_type: "RANGE".into(),
                },
            ],
            provisioned_throughput: Some(ProvisionedThroughput {
                read_capacity_units: 3,
                write_capacity_units: 3,
            }),
            ..Default::default()
        })
        .await
}

async fn create_or_empty_table(dynamo: &impl DynamoDb, table_name: impl Into<String>) {
    init_test_env();
    let table_name = table_name.into();

    let _ = dynamo.delete_table(DeleteTableInput {
        table_name: table_name.clone(),
    }).await;

    tokio::time::delay_for(Duration::from_millis(250)).await;

    while let Err(_e) = try_create_table(dynamo, table_name.clone()).await {
        tokio::time::delay_for(Duration::from_millis(250)).await;
    }
}

// Given an empty timeline
// When a canonical creation event comes in
// Then the newly created session should be in the timeline
#[quickcheck]
fn canon_create_on_empty_timeline(asset_id: String, pid: u64) {
    init_test_env();
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_canon_create_on_empty_timeline";
    let dynamo = DynamoDbClient::from_env();

    runtime
        .block_on(create_or_empty_table(&dynamo, table_name));

    let session_db = SessionDb::new(dynamo, table_name);

    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1544301484600,
        is_creation: true,
        negation_keys: HashMap::default()
    };

    let session_id = runtime
        .block_on(session_db.handle_unid_session(unid, false))
        .expect("Failed to create session");

    assert!(!session_id.is_empty());
}

// Given a timeline with a single session, where that session has a non canon
//      creation time 'X'
// When a canonical creation event comes in with a creation time of 'Y'
//      where 'Y' < 'X'
// Then the session should be updated to have 'Y' as its canonical create time
#[quickcheck]
fn canon_create_update_existing_non_canon_create(asset_id: String, pid: u64) {
    init_test_env();
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_canon_create_update_existing_non_canon_create";
    let dynamo = DynamoDbClient::from_env();

    runtime
        .block_on(create_or_empty_table(&dynamo, table_name));

    let session_db = SessionDb::new(dynamo, table_name);

    // Given a timeline with a single session, where that session has a non canon
    //      creation time 'X'
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_500,
        is_creation: true,
        negation_keys: HashMap::default()
    };
    let session = Session {
        pseudo_key: format!("{}{}", asset_id, pid),
        create_time: 1_544_301_484_600,
        is_create_canon: false,
        session_id: "SessionId".into(),
        is_end_canon: false,
        end_time: 1_544_301_484_700,
        version: 0,
    };

    runtime
        .block_on(session_db.create_session(&unid, &session))
        .expect("Failed to create session");

    // When a canonical creation event comes in with a creation time of 'Y'
    //      where 'Y' < 'X'
    let session_id = runtime
        .block_on(session_db.handle_unid_session(unid, false))
        .expect("Failed to handle unid");

    assert_eq!(session_id, "SessionId");
}

// Given a timeline with a single session, where that session has a non canon
//      creation time 'X'
// When a noncanonical creation event comes in with a creation time of 'Y'
//      where 'Y' < 'X'
// Then the session should be updated to have 'Y' as its noncanonical create time
#[quickcheck]
fn noncanon_create_update_existing_non_canon_create(asset_id: String, pid: u64) {
    init_test_env();
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_noncanon_create_update_existing_non_canon_create";
    let dynamo = DynamoDbClient::from_env();

    runtime
        .block_on(create_or_empty_table(&dynamo, table_name));

    let session_db = SessionDb::new(dynamo, table_name);
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_500,
        is_creation: false,
        negation_keys: HashMap::default()
    };

    // Given a timeline with a single session, where that session has a non canon
    //      creation time 'X'
    let session = Session {
        pseudo_key: format!("{}{}", asset_id, pid),
        create_time: 1_544_301_484_600,
        is_create_canon: false,
        session_id: "SessionId".into(),
        is_end_canon: false,
        end_time: 1_544_301_484_700,
        version: 0,
    };

    runtime
        .block_on(session_db.create_session(&unid, &session))
        .expect("Failed to create session");

    // When a noncanonical creation event comes in with a creation time of 'Y'
    //      where 'Y' < 'X'

    let session_id = runtime
        .block_on(session_db.handle_unid_session(unid, false))
        .expect("Failed to handle unid");

    // TODO: Assert that the create time was updated correctly
    assert_eq!(session_id, "SessionId");
}

// Given an empty timeline
// When a noncanon create event comes in and 'should_default' is true
// Then Create the new noncanon session
#[quickcheck]
fn noncanon_create_on_empty_timeline_with_default(asset_id: String, pid: u64) {
    init_test_env();
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_noncanon_create_on_empty_timeline_with_default";
    let dynamo = DynamoDbClient::from_env();

    runtime
        .block_on(create_or_empty_table(&dynamo, table_name));

    let session_db = SessionDb::new(dynamo, table_name);

    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_500,
        is_creation: false,
        negation_keys: HashMap::new(),
    };

    let session_id = runtime
        .block_on(session_db.handle_unid_session(unid, true))
        .expect("Failed to create session");

    assert!(!session_id.is_empty());
}

// Given an empty timeline
// When a noncanon create event comes in and 'should_default' is false
// Then return an error
#[test]
fn noncanon_create_on_empty_timeline_without_default() {
    init_test_env();
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_noncanon_create_on_empty_timeline_without_default";
    let dynamo = DynamoDbClient::from_env();

    runtime
        .block_on(create_or_empty_table(&dynamo, table_name));

    let session_db = SessionDb::new(dynamo, table_name);

    let unid = UnidSession {
        pseudo_key: "asset_id_a1234".into(),
        timestamp: 1_544_301_484_500,
        is_creation: false,
        negation_keys: HashMap::new(),
    };

    let session_id = runtime.block_on(session_db.handle_unid_session(unid, false));
    assert!(session_id.is_err());
}

#[quickcheck]
fn update_end_time(asset_id: String, pid: u64) {
    init_test_env();
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_update_end_time";
    let dynamo = DynamoDbClient::from_env();

    runtime
        .block_on(create_or_empty_table(&dynamo, table_name));

    let session_db = SessionDb::new(dynamo, table_name);
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_800,
        is_creation: false,
        negation_keys: HashMap::new(),
    };


    // Given a timeline with a single session, where that session has a non canon
    //      end time 'X'
    let session = Session {
        pseudo_key: format!("{}{}", asset_id, pid),
        create_time: 1_544_301_484_600,
        is_create_canon: false,
        session_id: "SessionId".into(),
        is_end_canon: false,
        end_time: 1_544_301_484_700,
        version: 0,
    };

    runtime
        .block_on(session_db.create_session(&unid, &session))
        .expect("Failed to create session");

    // When a canonical creation event comes in with an end time of 'Y'
    //      where 'Y' < 'X'

    let session_id = runtime
        .block_on(session_db.handle_unid_session(unid, false))
        .expect("Failed to handle unid");

    assert_eq!(session_id, "SessionId");
}

#[tokio::test]
async fn test_negation_keys_no_preexisting() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env();
    let asset_id = "test_negation_keyssomehostname";
    let pid = 1000u64;

    let table_name = "test_negation_keys";
    let dynamo = DynamoDbClient::from_env();

    create_or_empty_table(&dynamo, table_name).await;

    let session_db = SessionDb::new(dynamo, table_name);

    // Given a timeline with a single session, where that session has a non canon
    //      end time 'X'
    let session = Session {
        pseudo_key: format!("{}{}", asset_id, pid),
        create_time: 1_544_301_484_600,
        is_create_canon: false,
        session_id: "SessionId".into(),
        is_end_canon: false,
        end_time: 1_544_301_484_700,
        version: 0,
    };

    let negation_keys = hmap! {
        "parent_process_id".to_string() => ImmutableUintProp{prop: 1234}.into( )
    };
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_800,
        is_creation: false,
        negation_keys,
    };

    session_db.create_session(&unid, &session).await
        .expect("Failed to create session");

    // When a canonical creation event comes in with an end time of 'Y'
    //      where 'Y' < 'X'
    let session_id = session_db.handle_unid_session(unid, false).await
        .expect("Failed to handle unid");

    assert_eq!(session_id, "SessionId");
    Ok(())
}

#[tokio::test]
async fn test_negation_keys_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let asset_id = "test_negation_keys_mismatchsomehostname";
    let pid = 1000u64;

    let table_name = "test_negation_keys_mismatch";
    let dynamo = DynamoDbClient::from_env();

    create_or_empty_table(&dynamo, table_name).await;

    let session_db = SessionDb::new(dynamo, table_name);

    // Given a timeline with a single session, where that session has a non canon
    //      end time 'X'
    let session = Session {
        pseudo_key: format!("{}{}", asset_id, pid),
        create_time: 1_544_301_484_600,
        is_create_canon: false,
        session_id: "SessionId".into(),
        is_end_canon: false,
        end_time: 1_544_301_484_700,
        version: 0,
    };

    let negation_keys = hmap! {
        "parent_process_id".to_string() => ImmutableUintProp{prop: 1234}.into( )
    };
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_800,
        is_creation: false,
        negation_keys,
    };
    session_db.create_session(&unid, &session).await
        .expect("Failed to create session");

    let negation_keys = hmap! {
        "parent_process_id".to_string() => ImmutableUintProp{prop: 2345}.into( )
    };
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_800,
        is_creation: false,
        negation_keys,
    };
    // When an event comes in with a negation key that does not match the existing timeline,
    // we should expect it to miss
    assert!(session_db.handle_unid_session(unid, false).await.is_err());

    Ok(())
}


#[tokio::test]
async fn test_negation_keys_single_mismatch_single_match() -> Result<(), Box<dyn std::error::Error>> {
    let asset_id = "test_negation_keys_mismatchsomehostname";
    let pid = 1000u64;

    let table_name = "test_negation_keys_mismatch";
    let dynamo = DynamoDbClient::from_env();

    create_or_empty_table(&dynamo, table_name).await;

    let session_db = SessionDb::new(dynamo, table_name);

    // Given a timeline with a single session, where that session has a non canon
    //      end time 'X'
    let session = Session {
        pseudo_key: format!("{}{}", asset_id, pid),
        create_time: 1_544_301_484_600,
        is_create_canon: false,
        session_id: "SessionId".into(),
        is_end_canon: false,
        end_time: 1_544_301_484_700,
        version: 0,
    };

    let negation_keys = hmap! {
       "user_id".to_string() => ImmutableUintProp{prop: 1000}.into( ),
       "parent_process_id".to_string() => ImmutableUintProp{prop: 1234}.into( )
    };
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_800,
        is_creation: false,
        negation_keys,
    };
    session_db.create_session(&unid, &session).await
        .expect("Failed to create session");

    let negation_keys = hmap! {
        "user_id".to_string() => ImmutableUintProp{prop: 1000}.into( ),
        "parent_process_id".to_string() => ImmutableUintProp{prop: 2345}.into( )
    };
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_800,
        is_creation: false,
        negation_keys,
    };
    // When an event comes in with a negation key that does not match the existing timeline,
    // we should expect it to miss, even if another negation key does match
    assert!(session_db.handle_unid_session(unid, false).await.is_err());

    Ok(())
}
