use std::convert::TryFrom;

use failure::{
    bail,
    Error,
};
use hmap::hmap;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{
    AttributeValue,
    AttributeValueUpdate,
    Delete,
    DeleteItemInput,
    DynamoDb,
    Put,
    PutItemInput,
    QueryInput,
    TransactWriteItem,
    TransactWriteItemsInput,
    UpdateItemInput,
};
use tracing::{
    info,
    warn,
};
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

    #[tracing::instrument(skip(self, unid), err)]
    pub async fn find_first_session_after(
        &self,
        unid: &UnidSession,
    ) -> Result<Option<Session>, Error> {
        info!(message="Finding first session after", table_name=?&self.table_name);
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

        let res = self.dynamo.query(query).await;
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

    #[tracing::instrument(skip(self, unid), err)]
    pub async fn find_last_session_before(
        &self,
        unid: &UnidSession,
    ) -> Result<Option<Session>, Error> {
        info!(message = "Finding last session before");
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

        let res = self.dynamo.query(query).await?;

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
    #[tracing::instrument(skip(self, session), err)]
    pub async fn update_session_create_time(
        &self,
        session: &Session,
        new_time: u64,
        is_canon: bool,
    ) -> Result<(), Error> {
        info!(message = "Updating session create time");
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

        self.dynamo
            .transact_write_items(TransactWriteItemsInput {
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
            })
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, session), err)]
    pub async fn make_create_time_canonical(&self, session: &Session) -> Result<(), Error> {
        info!(message = "Updating session end time");
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

        self.dynamo.update_item(upd_req).await?;

        Ok(())
    }

    // Update version, and use it as a constraint
    #[tracing::instrument(skip(self, session), err)]
    pub async fn update_session_end_time(
        &self,
        session: &Session,
        new_time: u64,
        is_canon: bool,
    ) -> Result<(), Error> {
        info!(message = "Updating session end time");
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

        self.dynamo.update_item(upd_req).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, session), err)]
    pub async fn create_session(&self, session: &Session) -> Result<(), Error> {
        let put_req = PutItemInput {
            item: serde_dynamodb::to_hashmap(session).unwrap(),
            table_name: self.table_name.clone(),
            ..Default::default()
        };

        self.dynamo.put_item(put_req).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, session), err)]
    pub async fn delete_session(&self, session: &Session) -> Result<(), Error> {
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

        self.dynamo.delete_item(del_req).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, unid), err)]
    pub async fn handle_creation_event(&self, unid: UnidSession) -> Result<String, Error> {
        info!(
            message="Handling unid session creation",
            pseudo_key=?unid.pseudo_key, timestamp=?unid.timestamp
        );

        // Look for first session where session.create_time >= unid.create_time
        let session = self.find_first_session_after(&unid).await?;

        if let Some(session) = session {
            // If session.is_create_canon is false,
            // This means that there is a 'Guessed' session in the future,
            // and we should consider this the canonical ID for that session
            if !session.is_create_canon && session.create_time != unid.timestamp {
                info!(message = "Extending session create_time");
                self.update_session_create_time(&session, unid.timestamp, true)
                    .await?;
                return Ok(session.session_id);
            }

            tracing::debug!(
                "UNID: {} - {} - {}",
                unid.pseudo_key,
                unid.timestamp,
                unid.pseudo_key
            );

            // If the timestamps are the same, we've found the session_id
            // No need to update the database here - it's already canonical,
            // with an accurate timestamp
            if skewed_cmp(unid.timestamp, session.create_time) {
                info!(message = "Found existing session with exact create time");
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

        info!(message = "Creating session");
        self.create_session(&session).await?;
        Ok(session.session_id)
    }

    #[tracing::instrument(skip(self, unid), err)]
    pub async fn handle_last_seen(
        &self,
        unid: UnidSession,
        should_default: bool,
    ) -> Result<String, Error> {
        info!(
            message="Handling unid session",
            pseudo_key=?unid.pseudo_key, timestamp=?unid.timestamp
        );

        // Look for session where session.create_time <= unid.create_time <= session.end_time
        // Look for last session where session.create_time <= unid.create_time
        let session = self.find_last_session_before(&unid).await?;
        if let Some(mut session) = session {
            if unid.timestamp < session.end_time || skewed_cmp(unid.timestamp, session.end_time) {
                info!(message = "Identified session because it fell within a timeline.");
                return Ok(session.session_id);
            }

            if !session.is_end_canon {
                session.end_time = unid.timestamp;
                info!(message = "Updating session end_time.");
                //                self.update_session_end_time(&session, unid.timestamp, false)?;

                return Ok(session.session_id);
            }
        }

        let session = self.find_first_session_after(&unid).await?;
        if let Some(session) = session {
            if !session.is_create_canon {
                info!(message = "Found a later, non canonical session. Extending create_time.");

                self.update_session_create_time(&session, unid.timestamp, false)
                    .await?;
                return Ok(session.session_id);
            }
        }

        if should_default {
            info!(message = "Defaulting and creating new session.");
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
            warn!(message = "Could not attribute session. Not defaulting.");
            bail!(
                "Could not attribute session. should_default {}. Not defaulting.",
                should_default
            )
        }
    }

    #[tracing::instrument(skip(self), err)]
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
