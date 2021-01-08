use std::time::Duration;

use node_identifier::init_dynamodb_client;
use node_identifier::sessiondb::SessionDb;
use node_identifier::sessions::{Session, UnidSession};
use quickcheck_macros::quickcheck;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{
    AttributeDefinition, CreateTableError, CreateTableInput, CreateTableOutput, DeleteTableInput,
    DynamoDb, KeySchemaElement, ProvisionedThroughput,
};
use tokio::runtime::Runtime;

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

fn create_or_empty_table(dynamo: &impl DynamoDb, table_name: impl Into<String>) {
    let mut runtime = Runtime::new().unwrap();
    let table_name = table_name.into();

    let _ = runtime.block_on(dynamo.delete_table(DeleteTableInput {
        table_name: table_name.clone(),
    }));

    std::thread::sleep(Duration::from_millis(250));

    while let Err(_e) = runtime.block_on(try_create_table(dynamo, table_name.clone())) {
        std::thread::sleep(Duration::from_millis(250));
    }
}

// Given an empty timeline
// When a canonical creation event comes in
// Then the newly created session should be in the timeline
#[quickcheck]
#[cfg(feature = "integration")]
fn canon_create_on_empty_timeline(asset_id: String, pid: u64) {
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_canon_create_on_empty_timeline";
    let dynamo = init_dynamodb_client();

    create_or_empty_table(&dynamo, table_name);

    let session_db = SessionDb::new(dynamo, table_name);

    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1544301484600,
        is_creation: true,
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
#[cfg(feature = "integration")]
fn canon_create_update_existing_non_canon_create(asset_id: String, pid: u64) {
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_canon_create_update_existing_non_canon_create";
    let dynamo = init_dynamodb_client();

    create_or_empty_table(&dynamo, table_name);

    let session_db = SessionDb::new(dynamo, table_name);

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
        .block_on(session_db.create_session(&session))
        .expect("Failed to create session");

    // When a canonical creation event comes in with a creation time of 'Y'
    //      where 'Y' < 'X'
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_500,
        is_creation: true,
    };

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
#[cfg(feature = "integration")]
fn noncanon_create_update_existing_non_canon_create(asset_id: String, pid: u64) {
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_noncanon_create_update_existing_non_canon_create";
    let dynamo = init_dynamodb_client();

    create_or_empty_table(&dynamo, table_name);

    let session_db = SessionDb::new(dynamo, table_name);

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
        .block_on(session_db.create_session(&session))
        .expect("Failed to create session");

    // When a noncanonical creation event comes in with a creation time of 'Y'
    //      where 'Y' < 'X'
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_500,
        is_creation: false,
    };

    let session_id = runtime
        .block_on(session_db.handle_unid_session(unid, false))
        .expect("Failed to handle unid");

    // TODO: Assert that the create time was updated correctly
    assert_eq!(session_id, "SessionId");
}

// Given a timeline with two existing sessions, session A and session B
// where A.create_time = X and B.create_time = Y
// When a canonical creation event comes in with a creation time of 'Z'
//      where 'X' < 'Z' < 'Y'
// Then the new session should be created
#[test]
#[cfg(feature = "integration")]
fn canon_create_on_timeline_with_surrounding_canon_sessions() {
    let table_name = "process_history_canon_create_on_timeline_with_surrounding_canon_sessions";
    let dynamo = init_dynamodb_client();

    create_or_empty_table(&dynamo, table_name);

    let session_db = SessionDb::new(dynamo, table_name);
}

// Given an empty timeline
// When a noncanon create event comes in and 'should_default' is true
// Then Create the new noncanon session
#[quickcheck]
#[cfg(feature = "integration")]
fn noncanon_create_on_empty_timeline_with_default(asset_id: String, pid: u64) {
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_noncanon_create_on_empty_timeline_with_default";
    let dynamo = init_dynamodb_client();

    create_or_empty_table(&dynamo, table_name);

    let session_db = SessionDb::new(dynamo, table_name);

    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_500,
        is_creation: false,
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
#[cfg(feature = "integration")]
fn noncanon_create_on_empty_timeline_without_default() {
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_noncanon_create_on_empty_timeline_without_default";
    let dynamo = init_dynamodb_client();

    create_or_empty_table(&dynamo, table_name);

    let session_db = SessionDb::new(dynamo, table_name);

    let unid = UnidSession {
        pseudo_key: "asset_id_a1234".into(),
        timestamp: 1_544_301_484_500,
        is_creation: false,
    };

    let session_id = runtime.block_on(session_db.handle_unid_session(unid, false));
    assert!(session_id.is_err());
}

// Given a timeline with one session, where the session has a create_time
//      of X
// When a canon create event comes in with a create time within ~100ms of X
// Then we should make the session create time canonical
#[test]
#[cfg(feature = "integration")]
fn canon_create_on_timeline_with_existing_session_within_skew() {
    let table_name = "process_history_canon_create_on_timeline_with_existing_session_within_skew";
    let dynamo = init_dynamodb_client();

    create_or_empty_table(&dynamo, table_name);

    let session_db = SessionDb::new(dynamo, table_name);
}

#[quickcheck]
#[cfg(feature = "integration")]
fn update_end_time(asset_id: String, pid: u64) {
    let mut runtime = Runtime::new().unwrap();
    let table_name = "process_history_update_end_time";
    let dynamo = init_dynamodb_client();

    create_or_empty_table(&dynamo, table_name);

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

    runtime
        .block_on(session_db.create_session(&session))
        .expect("Failed to create session");

    // When a canonical creation event comes in with an end time of 'Y'
    //      where 'Y' < 'X'
    let unid = UnidSession {
        pseudo_key: format!("{}{}", asset_id, pid),
        timestamp: 1_544_301_484_800,
        is_creation: false,
    };

    let session_id = runtime
        .block_on(session_db.handle_unid_session(unid, false))
        .expect("Failed to handle unid");

    assert_eq!(session_id, "SessionId");
}
