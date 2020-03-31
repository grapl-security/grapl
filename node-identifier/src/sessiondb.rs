use failure::Error;
use futures::future::Future;
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{
    AttributeValue, AttributeValueUpdate, Condition, Delete, DeleteItemInput, DynamoDb,
    DynamoDbClient, ExpectedAttributeValue, GetItemInput, ListTablesInput, Put, PutItemInput,
    QueryError, QueryInput, TransactWriteItem, TransactWriteItemsInput, Update, UpdateItemInput,
};
use std::convert::TryFrom;
use std::time::Duration;
use uuid::Uuid;

use crate::sessions::*;

#[derive(Debug, Clone)]
pub struct SessionDb<D>
    where
        D: DynamoDb,
{
    dynamo: D,
    table_name: String,
}

impl<D> SessionDb<D>
    where
        D: DynamoDb,
{
    pub fn new(dynamo: D, table_name: impl Into<String>) -> Self {
        Self {
            dynamo,
            table_name: table_name.into(),
        }
    }

    pub async fn find_first_session_after(&self, unid: &UnidSession) -> Result<Option<Session>, Error> {
        info!("Finding first session after : {}", &self.table_name);
        let query = QueryInput {
            consistent_read: Some(true),
            limit: Some(1),
            table_name: self.table_name.clone(),
            key_condition_expression: Some(
                "pseudo_key = :pseudo_key AND create_time >= :create_time".into(),
            ),
            expression_attribute_values: Some(hmap! {
                ":pseudo_key".to_owned() => AttributeValue {
                    s: unid.pseudo_key.clone().into(),
                    ..Default::default()
                },
                ":create_time".to_owned() => AttributeValue {
                    n: unid.timestamp.to_string().into(),
                    ..Default::default()
                }
            }),
            ..Default::default()
        };

        let res = wait_on!(self.dynamo.query(query));
        if let Err(RusotoError::Unknown(ref e)) = res {
            bail!("Query failed with error: {:?}", e);
        };

        if let Some(items) = res?.items {
            match &items[..] {
                [] => Ok(None),
                [item] => Session::try_from(item.clone()).map(Option::from),
                _ => bail!("Unexpected number of items returned"),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn find_last_session_before(&self, unid: &UnidSession) -> Result<Option<Session>, Error> {
        info!("Finding last session before");
        let query = QueryInput {
            consistent_read: Some(true),
            limit: Some(1),
            scan_index_forward: Some(false),
            table_name: self.table_name.clone(),
            key_condition_expression: Some(
                "pseudo_key = :pseudo_key AND create_time <= :create_time".into(),
            ),
            expression_attribute_values: Some(hmap! {
                ":pseudo_key".to_owned() => AttributeValue {
                    s: unid.pseudo_key.clone().into(),
                    ..Default::default()
                },
                ":create_time".to_owned() => AttributeValue {
                    n: unid.timestamp.to_string().into(),
                    ..Default::default()
                }
            }),
            ..Default::default()
        };

        let res = wait_on!(self.dynamo.query(query))?;

        if let Some(items) = res.items {
            match &items[..] {
                [] => Ok(None),
                [item] => Session::try_from(item.clone()).map(Option::from),
                _ => bail!("Unexpected number of items returned"),
            }
        } else {
            Ok(None)
        }
    }

    // `create_time` is the sort key in the table, so updating it is not possible.
    // Instead, in one transaction, the row must be deleted and recreated with the
    // new create_time
    // This method assumes that the `session` passed in has already been modified
    pub async fn update_session_create_time(
        &self,
        session: &Session,
        new_time: u64,
        is_canon: bool,
    ) -> Result<(), Error> {
        info!("Updating session create time");
        let mut new_session = session.to_owned();
        new_session.create_time = new_time;
        new_session.is_create_canon = is_canon;
        new_session.version += 1;
        // Create new session with new create_time, increment version

        let put_req = Put {
            item: serde_dynamodb::to_hashmap(&new_session).unwrap(),
            table_name: self.table_name.clone(),
            ..Default::default()
        };

        let del_req = Delete {
            key: hmap! {
                "pseudo_key".to_owned() => AttributeValue {
                    s: session.pseudo_key.clone().into(),
                    ..Default::default()
                },
                "create_time".to_owned() => AttributeValue {
                    n: session.create_time.to_string().into(),
                    ..Default::default()
                }
            },
            table_name: self.table_name.clone(),
            ..Default::default()
        };

        wait_on!(self.dynamo.transact_write_items(TransactWriteItemsInput {
            transact_items: vec![
                TransactWriteItem {
                    delete: del_req.into(),
                    ..Default::default()
                },
                TransactWriteItem {
                    put: put_req.into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }))?;

        Ok(())
    }

    pub async fn make_create_time_canonical(
        &self,
        session: &Session,
    ) -> Result<(), Error> {
        info!("Updating session end time");
        // Use version as a constraint
        let upd_req = UpdateItemInput {
            key: hmap! {
                "pseudo_key".to_owned() => AttributeValue {
                    s: session.pseudo_key.clone().into(),
                    ..Default::default()
                },
                "create_time".to_owned() => AttributeValue {
                    n: session.create_time.to_string().into(),
                    ..Default::default()
                }
            },
            attribute_updates: Some(hmap! {
                "is_create_canon".to_owned() => AttributeValueUpdate {
                    value: Some(AttributeValue {
                            bool: true.into(),
                            ..Default::default()
                        }),
                    ..Default::default()
                },
                "version".to_owned() => AttributeValueUpdate {
                    value: Some(AttributeValue {
                        n: (session.version + 1).to_string().into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            }),
            table_name: self.table_name.clone(),
            condition_expression: Some("version = :version".into()),
            expression_attribute_values: Some(hmap! {
                ":version".to_owned() => AttributeValue {
                    n: session.version.to_string().into(),
                    ..Default::default()
                }
            }),
            ..Default::default()
        };

        wait_on!(self.dynamo.update_item(upd_req))?;

        Ok(())
    }

    // Update version, and use it as a constraint
    pub async fn update_session_end_time(
        &self,
        session: &Session,
        new_time: u64,
        is_canon: bool,
    ) -> Result<(), Error> {
        info!("Updating session end time");
        // Use version as a constraint
        let upd_req = UpdateItemInput {
            key: hmap! {
                "pseudo_key".to_owned() => AttributeValue {
                    s: session.pseudo_key.clone().into(),
                    ..Default::default()
                },
                "create_time".to_owned() => AttributeValue {
                    n: session.create_time.to_string().into(),
                    ..Default::default()
                }
            },
            attribute_updates: Some(hmap! {
                "end_time".to_owned() => AttributeValueUpdate {
                    value: Some(AttributeValue {
                        n: new_time.to_string().into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                "is_end_canon".to_owned() => AttributeValueUpdate {
                    value: Some(AttributeValue {
                            bool: is_canon.into(),
                            ..Default::default()
                        }),
                    ..Default::default()
                },
                "version".to_owned() => AttributeValueUpdate {
                    value: Some(AttributeValue {
                        n: (session.version + 1).to_string().into(),
                        ..Default::default()
                    }),
                    ..Default::default()

                }
            }),
            table_name: self.table_name.clone(),
            condition_expression: Some("version = :version".into()),
            expression_attribute_values: Some(hmap! {
                ":version".to_owned() => AttributeValue {
                    n: session.version.to_string().into(),
                    ..Default::default()
                }
            }),
            ..Default::default()
        };

        wait_on!(self.dynamo.update_item(upd_req))?;

        Ok(())
    }

    pub async fn create_session(&self, session: &Session) -> Result<(), Error> {
        info!("create session");
        let put_req = PutItemInput {
            item: serde_dynamodb::to_hashmap(session).unwrap(),
            table_name: self.table_name.clone(),
            ..Default::default()
        };

        wait_on!(self.dynamo.put_item(put_req))?;

        Ok(())
    }

    pub async fn delete_session(&self, session: &Session) -> Result<(), Error> {
        info!("delete session");
        let del_req = DeleteItemInput {
            key: hmap! {
                "pseudo_key".to_owned() => AttributeValue {
                    s: session.pseudo_key.clone().into(),
                    ..Default::default()
                },
                "create_time".to_owned() => AttributeValue {
                    n: session.create_time.to_string().into(),
                    ..Default::default()
                }
            },
            table_name: self.table_name.clone(),
            ..Default::default()
        };

        wait_on!(self.dynamo.delete_item(del_req))?;
        Ok(())
    }

    pub async fn handle_creation_event(&self, unid: UnidSession) -> Result<String, Error> {
        info!(
            "Handling unid session creation, pseudo_key: {:?} seen at: {}.",
            unid.pseudo_key, unid.timestamp
        );

        // Look for first session where session.create_time >= unid.create_time
        let session = self.find_first_session_after(&unid).await?;

        if let Some(session) = session {
            // If session.is_create_canon is false,
            // This means that there is a 'Guessed' session in the future,
            // and we should consider this the canonical ID for that session
            if !session.is_create_canon && session.create_time != unid.timestamp {
                info!("Extending session create_time");
                self.update_session_create_time(&session, unid.timestamp, true).await?;
                return Ok(session.session_id);
            }

            // If the timestamps are the same, we've found the session_id
            // No need to update the database here - it's already canonical,
            // with an accurate timestamp
            if skewed_cmp(unid.timestamp, session.create_time) {
                info!("Found existing session with exact create time");
                return Ok(session.session_id);
            }

            // We should never be looking at a case where the query returned
            // a create_time less than the unid.timestamp
            if unid.timestamp > session.create_time {
                bail!(
                    "unid.timestamp > session.create_time {} {}",
                    unid.timestamp,
                    session.create_time
                );
            }
        }

        // Look for last session where session.create_time <= unid.create_time
        let session = self.find_last_session_before(&unid).await?;

        if let Some(session) = session {
            // If session.end_time >= unid.create_time (indicates overlapping sessions, error)
            // This will correct that session so that it does not overlap anymore.
            if session.end_time >= unid.timestamp {
                warn!(
                    "Found session created before new session. Fixing overlapping end_time.
                    {:?}
                    {:?}
                ",
                    session, unid
                );
                // if session.end_time is NOT canonical, we can update it
                //                self.update_session_end_time(&session, unid.timestamp - 100, session.is_end_canon)?;
            }
        }

        // Create new session, return new session id
        let session = Session {
            session_id: Uuid::new_v4().to_string(),
            create_time: unid.timestamp,
            end_time: unid.timestamp + 101,
            is_create_canon: true,
            is_end_canon: false,
            version: 0,
            pseudo_key: unid.pseudo_key,
        };

        info!("Creating session");
        self.create_session(&session).await?;
        Ok(session.session_id)
    }

    pub async fn handle_last_seen(
        &self,
        unid: UnidSession,
        should_default: bool,
    ) -> Result<String, Error> {
        info!(
            "Handling unid session, pseudo_key: {:?} seen at: {}.",
            unid.pseudo_key, unid.timestamp
        );

        // Look for session where session.create_time <= unid.create_time <= session.end_time
        // Look for last session where session.create_time <= unid.create_time
        let session = self.find_last_session_before(&unid).await?;
        if let Some(mut session) = session {
            if unid.timestamp < session.end_time || skewed_cmp(unid.timestamp, session.end_time) {
                info!("Identified session because it fell within a timeline.");
                return Ok(session.session_id);
            }

            if !session.is_end_canon {
                session.end_time = unid.timestamp;
                info!("Updating session end_time.");
                //                self.update_session_end_time(&session, unid.timestamp, false)?;

                return Ok(session.session_id);
            }
        }

        let session = self.find_first_session_after(&unid).await?;
        if let Some(session) = session {
            if !session.is_create_canon {
                info!("Found a later, non canonical session. Extending create_time..");

                self.update_session_create_time(&session, unid.timestamp, false).await?;
                return Ok(session.session_id);
            }
        }

        if should_default {
            info!("Defaulting and creating new session.");
            let session_id = Uuid::new_v4().to_string();
            let session = Session {
                session_id: session_id.clone(),
                create_time: unid.timestamp,
                end_time: unid.timestamp + 101,
                is_create_canon: false,
                is_end_canon: false,
                version: 0,
                pseudo_key: unid.pseudo_key,
            };
            self.create_session(&session).await?;

            Ok(session_id)
        } else {
            warn!("Could not attribute session. Not defaulting.");
            bail!(
                "Could not attribute session. should_default {}. Not defaulting.",
                should_default
            )
        }
    }

    pub async fn handle_unid_session(
        &self,
        mut unid: UnidSession,
        should_default: bool,
    ) -> Result<String, Error> {
        unid.timestamp = shave_int(unid.timestamp, 1);
        if unid.is_creation {
            self.handle_creation_event(unid).await
        } else {
            self.handle_last_seen(unid, should_default).await
        }
    }
}

pub fn skewed_cmp(ts_1: u64, ts_2: u64) -> bool {
    ts_1 - 10 < ts_2 && ts_1 + 10 > ts_2
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusoto_dynamodb::KeySchemaElement;
    use rusoto_dynamodb::{
        AttributeDefinition, CreateTableError, CreateTableInput, DeleteTableInput,
        ProvisionedThroughput,
    };

    fn create_or_empty_table(dynamo: &impl DynamoDb, table_name: impl Into<String>) {
        let table_name = table_name.into();

        let _ = dynamo
            .delete_table(DeleteTableInput {
                table_name: table_name.clone(),
            })
            .with_timeout(Duration::from_secs(1))
            .sync();

        std::thread::sleep(Duration::from_millis(155));

        let res = dynamo
            .create_table(CreateTableInput {
                table_name: table_name.clone(),
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
            .with_timeout(Duration::from_secs(1))
            .sync()
            .expect("Failed to crate table");
    }

    fn local_dynamo() -> impl DynamoDb {
        let region = Region::Custom {
            endpoint: "http://localhost:8001".to_owned(),
            name: "us-east-9".to_owned(),
        };

        DynamoDbClient::new(region)
    }

    // Given an empty timeline
    // When a canonical creation event comes in
    // Then the newly created session should be in the timeline
    #[quickcheck]
    fn canon_create_on_empty_timeline(asset_id: String, pid: u64) {
        let table_name = "process_history_canon_create_on_empty_timeline";
        let dynamo = local_dynamo();

        create_or_empty_table(&dynamo, table_name);

        let session_db = SessionDb::new(&dynamo, table_name);

        let unid = UnidSession {
            pseudo_key: format!("{}{}", asset_id, pid),
            timestamp: 1544301484600,
            is_creation: true,
        };

        let session_id = session_db
            .handle_unid_session(unid, false)
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
        let table_name = "process_history_canon_create_update_existing_non_canon_create";
        let dynamo = local_dynamo();

        create_or_empty_table(&dynamo, table_name);

        let session_db = SessionDb::new(&dynamo, table_name);

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

        session_db
            .create_session(&session)
            .expect("Failed to create session");

        // When a canonical creation event comes in with a creation time of 'Y'
        //      where 'Y' < 'X'
        let unid = UnidSession {
            pseudo_key: format!("{}{}", asset_id, pid),
            timestamp: 1_544_301_484_500,
            is_creation: true,
        };

        let session_id = session_db
            .handle_unid_session(unid, false)
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
        let table_name = "process_history_noncanon_create_update_existing_non_canon_create";
        let dynamo = local_dynamo();

        create_or_empty_table(&dynamo, table_name);

        let session_db = SessionDb::new(&dynamo, table_name);

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

        session_db
            .create_session(&session)
            .expect("Failed to create session");

        // When a noncanonical creation event comes in with a creation time of 'Y'
        //      where 'Y' < 'X'
        let unid = UnidSession {
            pseudo_key: format!("{}{}", asset_id, pid),
            timestamp: 1_544_301_484_500,
            is_creation: false,
        };

        let session_id = session_db
            .handle_unid_session(unid, false)
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
    fn canon_create_on_timeline_with_surrounding_canon_sessions() {
        let table_name = "process_history_canon_create_on_timeline_with_surrounding_canon_sessions";
        let dynamo = local_dynamo();

        create_or_empty_table(&dynamo, table_name);

        let session_db = SessionDb::new(&dynamo, table_name);
    }

    // Given an empty timeline
    // When a noncanon create event comes in and 'should_default' is true
    // Then Create the new noncanon session
    #[quickcheck]
    fn noncanon_create_on_empty_timeline_with_default(asset_id: String, pid: u64) {
        let table_name = "process_history_noncanon_create_on_empty_timeline_with_default";
        let dynamo = local_dynamo();

        create_or_empty_table(&dynamo, table_name);

        let session_db = SessionDb::new(&dynamo, table_name);

        let unid = UnidSession {
            pseudo_key: format!("{}{}", asset_id, pid),
            timestamp: 1_544_301_484_500,
            is_creation: false,
        };

        let session_id = session_db
            .handle_unid_session(unid, true)
            .expect("Failed to create session");

        assert!(!session_id.is_empty());
    }

    // Given an empty timeline
    // When a noncanon create event comes in and 'should_default' is false
    // Then return an error
    #[test]
    fn noncanon_create_on_empty_timeline_without_default() {
        let table_name = "process_history_noncanon_create_on_empty_timeline_without_default";
        let dynamo = local_dynamo();

        create_or_empty_table(&dynamo, table_name);

        let session_db = SessionDb::new(&dynamo, table_name);

        let unid = UnidSession {
            pseudo_key: "asset_id_a1234".into(),
            timestamp: 1_544_301_484_500,
            is_creation: false,
        };

        let session_id = session_db.handle_unid_session(unid, false);
        assert!(session_id.is_err());
    }

    // Given a timeline with one session, where the session has a create_time
    //      of X
    // When a canon create event comes in with a create time within ~100ms of X
    // Then we should make the session create time canonical
    #[test]
    fn canon_create_on_timeline_with_existing_session_within_skew() {
        let table_name =
            "process_history_canon_create_on_timeline_with_existing_session_within_skew";
        let dynamo = local_dynamo();

        create_or_empty_table(&dynamo, table_name);

        let session_db = SessionDb::new(&dynamo, table_name);
    }

    #[quickcheck]
    fn update_end_time(asset_id: String, pid: u64) {
        let table_name = "process_history_update_end_time";
        let dynamo = local_dynamo();

        create_or_empty_table(&dynamo, table_name);

        let session_db = SessionDb::new(&dynamo, table_name);

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

        session_db
            .create_session(&session)
            .expect("Failed to create session");

        // When a canonical creation event comes in with an end time of 'Y'
        //      where 'Y' < 'X'
        let unid = UnidSession {
            pseudo_key: format!("{}{}", asset_id, pid),
            timestamp: 1_544_301_484_800,
            is_creation: false,
        };

        let session_id = session_db
            .handle_unid_session(unid, false)
            .expect("Failed to handle unid");

        assert_eq!(session_id, "SessionId");
    }
}
