use std::convert::TryFrom;

use failure::{
    bail,
    Error,
};
use hmap::hmap;
use log::{
    info,
    warn,
};
use rusoto_core::RusotoError;
use rusoto_dynamodb::{AttributeValue, AttributeValueUpdate, Delete, DeleteItemInput, Put, PutItemInput, QueryInput, TransactWriteItem, TransactWriteItemsInput, UpdateItemInput, WriteRequest, PutRequest, BatchWriteItemInput};
use uuid::Uuid;

use crate::sessions::*;
use grapl_graph_descriptions::NodeDescription;
use crate::dynamic_sessiondb::AttributedNode;
use itertools::{Itertools, Either};
use grapl_utils::rusoto_ext::dynamodb::GraplDynamoDbClientExt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SessionDb<D>
where
    D: GraplDynamoDbClientExt,
{
    dynamo: D,
    table_name: String,
}

impl<D> SessionDb<D>
where
    D: GraplDynamoDbClientExt,
{
    pub fn new(dynamo: D, table_name: impl Into<String>) -> Self {
        Self {
            dynamo,
            table_name: table_name.into(),
        }
    }

    pub async fn find_first_session_after(
        &self,
        unid: &UnidSession,
    ) -> Result<Option<Session>, Error> {
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

    pub async fn find_last_session_before(
        &self,
        unid: &UnidSession,
    ) -> Result<Option<Session>, Error> {
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

    pub async fn make_create_time_canonical(&self, session: &Session) -> Result<(), Error> {
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

        self.dynamo.update_item(upd_req).await?;

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

        self.dynamo.update_item(upd_req).await?;

        Ok(())
    }

    pub async fn create_session(&self, session: &Session) -> Result<(), Error> {
        info!("create session");
        let put_req = PutItemInput {
            item: serde_dynamodb::to_hashmap(session).unwrap(),
            table_name: self.table_name.clone(),
            ..Default::default()
        };

        self.dynamo.put_item(put_req).await?;

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

        self.dynamo.delete_item(del_req).await?;
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

                self.update_session_create_time(&session, unid.timestamp, false)
                    .await?;
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

    /// Given a collection of sessions, attempt to create all them.
    async fn create_sessions(
        &self,
        sessions: Vec<Session>
    ) -> Result<(), Error> {
        let write_operations: Vec<_> = sessions.into_iter()
            .map(|session| {
                PutRequest {
                    item: serde_dynamodb::to_hashmap(&session).unwrap()
                }
            })
            .map(|put| {
                WriteRequest {
                    delete_request: None,
                    put_request: Some(put)
                }
            })
            .collect();

        let mut batch_write_operation = HashMap::new();
        batch_write_operation.insert(self.table_name.clone(), write_operations);

        let batch_write_input = BatchWriteItemInput {
            request_items: batch_write_operation,
            return_consumed_capacity: None,
            return_item_collection_metrics: None
        };

        self.dynamo.batch_write_item_reliably(batch_write_input).await?;

        Ok(())
    }

    /// This function will take the [`UnidSessionNode`]s, create queries based on the passed in function,
    /// and then return what happens when attempting to query DynamoDB and convert those into [`Session`] objects.
    async fn find_sessions_from_queries<F>(
        &self,
        unid_session_nodes: Vec<UnidSessionNode>,
        create_query: F
    ) -> Result<Vec<(UnidSessionNode, Option<Session>)>, Error>
        where
            F: Fn(&UnidSession) -> QueryInput
    {
        let unid_queries: Vec<_> = unid_session_nodes.iter()
            .map(|UnidSessionNode(_, unid_session)| create_query(unid_session))
            .collect();

        let query_results = self.dynamo.batch_query(unid_queries).await?;

        let identification_results = query_results.into_iter()
            // if a single result was returned, attempt to convert it to a Session
            .map(|result| {
                let items = result.items?;

                match &items[..] {
                    [session_candidate] => Session::try_from(session_candidate.clone()).ok(),
                    _ => None
                }
            })
            // zip up the Sessions with the original UnidSessionNodes
            .zip(unid_session_nodes)
            .map(|(session, unid)| (unid, session))
            .collect();

        Ok(identification_results)
    }

    async fn find_first_sessions_after(
        &self,
        creation_unids: Vec<UnidSessionNode>
    ) -> Result<Vec<(UnidSessionNode, Option<Session>)>, Error> {
        let create_query_for_after_timestamp = |unid_session: &UnidSession| {
            QueryInput {
                consistent_read: Some(true),
                limit: Some(1),
                table_name: self.table_name.clone(),
                key_condition_expression: Some(
                    "pseudo_key = :pseudo_key AND create_time >= :create_time".into(),
                ),
                expression_attribute_values: Some(hmap! {
                        ":pseudo_key".to_owned() => AttributeValue {
                            s: unid_session.pseudo_key.clone().into(),
                            ..Default::default()
                        },
                        ":create_time".to_owned() => AttributeValue {
                            n: unid_session.timestamp.to_string().into(),
                            ..Default::default()
                        }
                    }),
                ..Default::default()
            }
        };

        self.find_sessions_from_queries(creation_unids, create_query_for_after_timestamp).await
    }

    async fn find_last_sessions_before(
        &self,
        last_seen_unids: Vec<UnidSessionNode>
    ) -> Result<Vec<(UnidSessionNode, Option<Session>)>, Error> {
        let create_query_for_before_timestamp = |unid_session: &UnidSession| {
            QueryInput {
                consistent_read: Some(true),
                limit: Some(1),
                scan_index_forward: Some(false),
                table_name: self.table_name.clone(),
                key_condition_expression: Some(
                    "pseudo_key = :pseudo_key AND create_time <= :create_time".into(),
                ),
                expression_attribute_values: Some(hmap! {
                        ":pseudo_key".to_owned() => AttributeValue {
                            s: unid_session.pseudo_key.clone().into(),
                            ..Default::default()
                        },
                        ":create_time".to_owned() => AttributeValue {
                            n: unid_session.timestamp.to_string().into(),
                            ..Default::default()
                        }
                    }),
                ..Default::default()
            }
        };

        self.find_sessions_from_queries(last_seen_unids, create_query_for_before_timestamp).await
    }

    fn split_session_pairs(&self, session_results: Vec<(UnidSessionNode, Option<Session>)>) -> (
        Vec<(UnidSessionNode, Session)>,
        Vec<UnidSessionNode>
    ) {
        session_results.into_iter()
            .partition_map(|(session_node, session)| {
                match session {
                    Some(session) => Either::Left((session_node, session)),
                    None => Either::Right(session_node)
                }
            })
    }

    async fn process_first_after_session_nodes(
        &self,
        first_after_session_pairs: Vec<(UnidSessionNode, Session)>
    ) -> Result<Vec<AttributedNode>, Error> {
        unimplemented!()
    }

    async fn process_last_before_session_nodes(
        &self,
        last_before_session_pairs: Vec<(UnidSessionNode, Session)>
    ) -> Result<Vec<AttributedNode>, Error> {
        unimplemented!()
    }

    async fn handle_creation_unid_sessions(
        &self,
        creation_unids: Vec<UnidSessionNode>
    ) -> Result<Vec<AttributedNode>, Error> {
        let first_after_session_results = self.find_first_sessions_after(creation_unids).await?;

        let (
            first_after_session_pairs,
            no_session_nodes
        ) = self.split_session_pairs(first_after_session_results);

        let first_after_attributed_nodes = self.process_first_after_session_nodes(first_after_session_pairs).await?;

        let (
            last_before_attributed_nodes,
            no_session_nodes
        ) = match no_session_nodes.is_empty() {
            true => {
                let last_before_session_results = self.find_last_sessions_before(no_session_nodes).await?;

                let (
                    last_before_session_pairs,
                    no_session_nodes
                ) = self.split_session_pairs(last_before_session_results);

                let last_before_attributed_nodes = self.process_last_before_session_nodes(last_before_session_pairs).await?;

                (last_before_attributed_nodes, no_session_nodes)
            },
            false => (vec![], vec![])
        };


        // TODO: process last_after sessions

        let (
            newly_attributed_nodes,
            sessions
        ): (Vec<_>, Vec<_>) = no_session_nodes.into_iter()
            .map(|UnidSessionNode(mut node_desc, unid_session)| {
                let old_key = node_desc.clone_node_key();
                let session = Session {
                    session_id: Uuid::new_v4().to_string(),
                    create_time: unid_session.timestamp,
                    end_time: unid_session.timestamp + 101,
                    is_create_canon: true,
                    is_end_canon: false,
                    version: 0,
                    pseudo_key: unid_session.pseudo_key,
                };

                // set the new, canonical node_key for our previously unidentified node
                node_desc.node_key = session.session_id.clone();

                let attributed_node = AttributedNode::new(node_desc, old_key);

                (attributed_node, session)
            }).unzip();

        if !sessions.is_empty() {
            self.create_sessions(sessions).await?;
        }

        let all_results: Vec<_> = newly_attributed_nodes.into_iter()
            .chain(last_before_attributed_nodes)
            .chain(first_after_attributed_nodes)
            .collect();

        Ok(all_results)
    }

    async fn handle_last_seen_unid_sessions(
        &self,
        last_seen_unids: Vec<UnidSessionNode>,
        should_default: bool
    ) -> Result<Vec<AttributedNode>, Error> {
        unimplemented!()
    }

    pub async fn identify_unid_session_nodes(
        &self,
        unids: Vec<UnidSessionNode>,
        should_default: bool
    ) -> Result<Vec<AttributedNode>, Error> {
        let (
            creation_unids,
            last_seen_unids
        ): (Vec<_>, Vec<_>) = unids.into_iter()
            .map(|mut entry | {
                let UnidSessionNode(_, unid) = &mut entry;
                unid.timestamp = shave_int(unid.timestamp, 1);

                entry
            })
            .partition(|UnidSessionNode(_, unid)| unid.is_creation);

        let identified_creation_nodes = self.handle_creation_unid_sessions(creation_unids).await?;
        let identified_last_seen_nodes = self.handle_last_seen_unid_sessions(last_seen_unids, should_default).await?;

        let identified_nodes = identified_creation_nodes.into_iter()
            .chain(identified_last_seen_nodes)
            .collect();

        Ok(identified_nodes)
    }
}

pub fn skewed_cmp(ts_1: u64, ts_2: u64) -> bool {
    ts_1 - 10 < ts_2 && ts_1 + 10 > ts_2
}

pub(crate) struct UnidSessionNode(NodeDescription, UnidSession);

impl UnidSessionNode {
    pub(crate) fn new(node: NodeDescription, unid_session: UnidSession) -> Self {
        Self(node, unid_session)
    }
}