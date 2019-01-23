use failure::Error;
use graph_descriptions::*;
use graph_descriptions::graph_description::*;
use mysql::{Pool, Transaction};
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::str;
use uuid;
use mysql::IsolationLevel;

use stopwatch::Stopwatch;
use cache::IdentityCache;
use session::{Session, Action};

pub fn find_first_session_after(conn: &mut Transaction, session: & impl Session) -> Result<Option<(String, bool)>, Error> {
    let query = format!(r#"
       SELECT session_id, create_time, is_guess
           FROM {}
       WHERE {} = "{key}" AND asset_id = "{asset_id}"
             AND create_time >= {create_time}
       ORDER BY create_time ASC
       LIMIT 1
       "#,
                        session.get_table_name(),
                        session.get_key_name(),
                        key=session.get_key().as_ref().replace("\\", "\\\\")
                            .replace("\"", "\\\"")
                            .replace("\'", "\\\'")
                            .replace("\n", "\\\n")
                            .replace("\t", "\\\t"),
                        asset_id=session.get_asset_id(),
                        create_time=session.get_timestamp()
    );

    let query_results = conn.prep_exec(&query, &())?;

    let row = query_results.into_iter().next();

    if row.is_none() {
        return Ok(None)
    }
    let row = row.unwrap();

    let row = row.as_ref().expect("Failed to unwrap row");
    let session_id: String = row.get("session_id").expect("is_guess");
    let is_guess: bool = row.get("is_guess").expect("is_guess");

    Ok(Some((session_id, is_guess)))
}

pub fn find_latest_guess(conn: &mut Transaction, session: & impl Session) -> Result<Option<String>, Error> {

    // given the timestamp for session
    // find the FIRST ID after that session
    // If it is a guess, we can assume that session's ID
    // If it is not a guess, we must create the id

    let query = format!(r#"
       SELECT session_id, create_time, is_guess
           FROM {}
       WHERE {} = "{key}" AND asset_id = "{asset_id}"
             AND create_time >= {create_time}
       ORDER BY create_time ASC
       LIMIT 1
       "#,
                        session.get_table_name(),
                        session.get_key_name(),
                        key=session.get_key().as_ref().replace("\\", "\\\\")
                            .replace("\"", "\\\"")
                            .replace("\'", "\\\'")
                            .replace("\n", "\\\n")
                            .replace("\t", "\\\t"),
                        asset_id=session.get_asset_id(),
                        create_time=session.get_timestamp()
    );

    info!("Query is: {}", &query);

    let query_results = conn.prep_exec(&query, &())?;

    info!("get_session_id prep_exec");

    let query_results: Vec<_> = query_results.collect();

    for row in &query_results {
        info!("Row {:#?}", row);
        let row = row.as_ref().expect("Failed to unwrap row");
        let is_guess: bool = row.get("is_guess").expect("is_guess");

        if is_guess {
            return Ok(Some(row.get("session_id").expect("session_id")));
        }

    }

    if !query_results.is_empty() {
        info!("Retrieving session id for latest session");
        let row = query_results.last().unwrap();
        let row = row.as_ref().expect("Failed to unwrap row");

        return Ok(Some(row.get("session_id").expect("session_id")));
    }

    info!("Went through all query results");

    Ok(None)
}

pub fn get_session_id(conn: &mut Transaction, session: & impl Session) -> Result<Option<String>, Error> {

    let maybe_id = check_exact_session(conn, session)?;

    if let Some(session_id) = maybe_id {
        return Ok(Some(session_id))
    }

    info!("get session id");
    let query = format!(r#"
       SELECT session_id, create_time
           FROM {}
       WHERE {} = "{key}" AND asset_id = "{asset_id}"
             AND create_time <= {create_time}
       ORDER BY create_time DESC
       "#,
        session.get_table_name(),
        session.get_key_name(),
        key=session.get_key().as_ref().replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\'", "\\\'")
            .replace("\n", "\\\n")
            .replace("\t", "\\\t"),
        asset_id=session.get_asset_id(),
        create_time=session.get_timestamp()
    );

    info!("Query is: {}", &query);

    let query_results = conn.prep_exec(&query, &())?;

    info!("get_session_id prep_exec");

    let query_results: Vec<_> = query_results.collect();

    for row in &query_results {
        info!("Row {:#?}", row);
        let row = row.as_ref().expect("Failed to unwrap row");
        let a_time: i64 = row.get("create_time").expect("create_time");
        let a_time: u64 = a_time as u64;

        if session.get_timestamp() >= a_time {
            return Ok(Some(row.get("session_id").expect("create_time")));
        }
    }

    if !query_results.is_empty() {
        info!("Retrieving session id for latest session");
        let row = query_results.last().unwrap();
        let row = row.as_ref().expect("Failed to unwrap row");

        return Ok(Some(row.get("session_id").expect("session_id")));
    }

    info!("Went through all query results");

    Ok(None)
}

pub fn check_exact_session(conn: &mut Transaction, session: & impl Session) -> Result<Option<String>, Error> {

    let min_ts = session.get_timestamp() - 100;
    let max_ts = session.get_timestamp() + 100;

    // TODO: We can probably add a bit of skew here, +/- 5 seconds would be safe
    let query = format!("
       SELECT session_id
       FROM {}
       WHERE {} = \"{}\" AND asset_id = \"{}\"
             AND create_time >= {}
             AND create_time <= {}
       LIMIT 1
             ",
        session.get_table_name(),session.get_key_name(), session.get_key().replace("\\", "\\\\")
                            .replace("\"", "\\\"")
                            .replace("\'", "\\\'")
                            .replace("\n", "\\\n")
                            .replace("\t", "\\\t"),
                        session.get_asset_id(),
                        min_ts,
                        max_ts
    );

    let query_result = conn.prep_exec(&query, &())?;

    let row = query_result.into_iter().next();

    if let Some(row) = row {
        return Ok(Some(row?.get("session_id").expect("session_id")));
    }

    Ok(None)
}

pub fn create_session(conn: &mut Transaction, session: & impl Session, is_guess: bool) -> Result<String, Error> {
    info!("create session id");

    // Check if we've already processed a session start with these exact values
    let maybe_id = check_exact_session(conn, session)?;

    if let Some(session_id) = maybe_id {
        return Ok(session_id)
    }

    // If we aren't guessing, we still need to check to see if we've guessed before, and if we're
    // about to create a new session when we should instead just take the ID of a guess
    let session_id = if !is_guess {
        // Find the next session. If it's a guess, create our session with that ID
        // Otherwise, generate a new ID
        let first_after = find_first_session_after(conn, session)?;

        if let Some((session_id, was_guessed)) = first_after {
            if was_guessed {
                session_id
            } else {
                uuid::Uuid::new_v4().to_string()
            }
        } else {
            uuid::Uuid::new_v4().to_string()
        }
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let query = format!("
       INSERT INTO {}
          (session_id, {}, asset_id, create_time, is_guess)
          VALUES
              (\"{}\", \"{}\", \"{}\", {}, {})",
        session.get_table_name(),
        session.get_key_name(), session_id, session.get_key().replace("\\", "\\\\")
                            .replace("\"", "\\\"")
                            .replace("\'", "\\\'")
                            .replace("\n", "\\\n")
                            .replace("\t", "\\\t"),
                        session.get_asset_id(),
                        session.get_timestamp(),
        is_guess
    );

    info!("create_session prep_exec {}", &query);
    let res = conn.prep_exec(&query, &());
    if let Err(ref e) = res {
        error!("{:#?}", e);
    }
    res?;

    Ok(session_id)
}


pub fn update_or_create(conn: &mut Transaction,
                    session: & impl Session,
                    should_default: bool
) -> Result<String, Error> {
    info!("update or create session id");

    let session_id = log_time!{
        "get_session_id",
        get_session_id(
            conn, session
        )?
    };

    if let Some(session_id) = session_id {
        info!("Got session_id");
        return Ok(session_id)
    }

    if should_default {
        info!("Did not get session id. Creating session_id");
        let latest_guess = find_latest_guess(conn, session)?;

        match latest_guess {
            Some(session_id) => {
                info!("Guessed session_id");
                Ok(session_id)
            }
            None => {
                info!("Creating session, defaulting");
                create_session(conn, session, true)
            }
        }
    } else {
        bail!("Failed to get the session id, did not default.")
    }
}

pub fn create_process_table(conn: &Pool) {
    info!("Creating process_history  table");
//    conn.prep_exec("DROP TABLE IF EXISTS `process_history`", &());

    conn.prep_exec("CREATE TABLE IF NOT EXISTS process_history (
                    primary_key     SERIAL PRIMARY KEY,
                    session_id      TEXT NOT NULL,
                    asset_id        TEXT NOT NULL,
                    pid             NUMERIC NOT NULL,
                    create_time     BIGINT UNSIGNED NOT NULL,
                    is_guess        BOOLEAN NOT NULL
                  )", &()).expect("process_history::create_table");
}

pub fn create_file_table(conn: &Pool) {
    info!("Creating file_history  table");
//    conn.prep_exec("DROP TABLE IF EXISTS `file_history`", &());

    conn.prep_exec("CREATE TABLE IF NOT EXISTS file_history (
                    primary_key     SERIAL PRIMARY KEY,
                    session_id      TEXT NOT NULL,
                    asset_id        TEXT NOT NULL,
                    path            TEXT NOT NULL,
                    create_time     BIGINT UNSIGNED NOT NULL,
                    is_guess        BOOLEAN NOT NULL
                  )", &()).expect("file_history::create_table");
}

pub fn create_connection_table(conn: &Pool) {
    info!("Creating connection_history  table");
//    conn.prep_exec("DROP TABLE IF EXISTS `connection_history`", &());

    conn.prep_exec("CREATE TABLE IF NOT EXISTS connection_history (
                    primary_key     SERIAL PRIMARY KEY,
                    session_id      TEXT NOT NULL,
                    asset_id        TEXT NOT NULL,
                    dir_port_ip     TEXT NOT NULL,
                    create_time     BIGINT UNSIGNED NOT NULL,
                    is_guess        BOOLEAN NOT NULL
                  )", &()).expect("connection_history::create_table");
}

pub fn attribute_session_node(conn: &mut Transaction,
                              node: impl Session,
                              should_default: bool,
) -> Result<String, Error> {

    let session_id = match node.get_action() {
        Action::Create => {
            info!("Handling created session {}", node.get_table_name());

            log_time!{
                "create_session",
                create_session(
                    conn, &node, false
                )?
            }
        },
        Action::UpdateOrCreate => {
            info!("Handling existing session");
            log_time!{
                "update_or_create",
                update_or_create(
                    conn,
                    &node,
                    should_default
                )?
            }
        },
        Action::Terminate => {
            warn!("Unimplemented!: Handling terminated session {:#?}", node);
//            let session_id = get_process_session_id(
//                conn, node.pid, node.asset_id(), node.timestamp
//            )?;
//
            unimplemented!("Terminate not implemented")
        },
    };

    Ok(session_id)
}

pub fn map_session_ids_to_graph(conn: &Pool,
                                key_map: &mut HashMap<String, String>,
                                dead_node_keys: &mut HashSet<String>,
                                unid_subgraph: &GraphDescription,
                                output_subgraph: &mut GraphDescription,
                                should_default: bool,
                                cache: IdentityCache,
) -> Result<(), Error> {

    // Maps old session ids to new ones
    let mut result = Ok(());

    for _node in unid_subgraph.nodes.clone() {
        let node: NodeDescription = _node.1.into();
        match node.which() {
            Node::ProcessNode(mut node) => {


                info!("Mapping session id for ProcessNode. pid {}", node.pid);
                let old_id = node.clone_key();

                let session_id = cache.check_cache(&node)?;
                if let Some(new_node_key) = session_id {
                    node.set_key(new_node_key.clone());
                    key_map.insert(old_id, new_node_key.clone());
                    output_subgraph.nodes.insert(new_node_key, node.into());
                    continue
                }

                let mut tx = conn.start_transaction(
                    false,
                    Some(IsolationLevel::Serializable),
                    Some(false)
                ).expect("Failed to acquire transaction");

                let attribution_res = attribute_session_node(&mut tx, &node, should_default);

                match attribution_res {
                    Ok(new_node_key) => {
                        info!("Successfully attributed session for ProcessNode");
                        tx.commit().expect("transaction commit failed");
                        node.set_key(new_node_key.clone());
                        key_map.insert(old_id, new_node_key.clone());
                        output_subgraph.nodes.insert(new_node_key.clone(), node.clone().into());
                        cache.update_cache(&node, new_node_key);
                    }
                    Err(e) => {
                        tx.rollback().expect("transaction rollback failed");
                        error!("Process Attribution Failure {:#?}", e);
                        dead_node_keys.insert(old_id);
                        result = Err(e);
                    }
                }
            }
            Node::FileNode(mut node) => {
                info!("Mapping sesion id for FileNode");
                let old_id = node.clone_key();

                let session_id = cache.check_cache(&node)?;
                if let Some(new_node_key) = session_id {
                    node.set_key(new_node_key.clone());
                    key_map.insert(old_id, new_node_key.clone());
                    output_subgraph.nodes.insert(new_node_key, node.into());
                    continue
                }

                let mut tx = conn.start_transaction(
                    false,
                    Some(IsolationLevel::Serializable),
                    Some(false)
                ).expect("Failed to acquire transaction");

                let attribution_res = attribute_session_node(&mut tx, &node, should_default);

                match attribution_res {
                    Ok(new_node_key) => {
                        info!("Successfully attributed session for FileNode");
                        tx.commit().expect("transaction commit failed");
                        node.set_key(new_node_key.clone());
                        key_map.insert(old_id, new_node_key.clone());
                        output_subgraph.nodes.insert(new_node_key.clone(), node.clone().into());
                        cache.update_cache(&node, new_node_key);
                    }
                    Err(e) => {
                        tx.rollback().expect("transaction rollback failed");
                        error!("Process Attribution Failure {:#?}", e);
                        dead_node_keys.insert(old_id);
                        result = Err(e);
                    }
                }
            }
            Node::OutboundConnectionNode(mut node) => {
                info!("Mapping sesion id for OutboundConnectionNode");
                let old_id = node.clone_key();

                let session_id = cache.check_cache(&node)?;
                if let Some(new_node_key) = session_id {
                    node.set_key(new_node_key.clone());
                    key_map.insert(old_id, new_node_key.clone());
                    output_subgraph.nodes.insert(new_node_key, node.into());
                    continue
                }

                let mut tx = conn.start_transaction(
                    false,
                    Some(IsolationLevel::Serializable),
                    Some(false)
                ).expect("Failed to acquire transaction");

                let attribution_res = attribute_session_node(&mut tx, &node, should_default);

                match attribution_res {
                    Ok(new_node_key) => {
                        info!("Successfully attributed session for OutboundConnectionNode");
                        tx.commit().expect("transaction commit failed");
                        node.set_key(new_node_key.clone());
                        key_map.insert(old_id, new_node_key.clone());
                        output_subgraph.nodes.insert(new_node_key.clone(), node.clone().into());
                        cache.update_cache(&node, new_node_key);
                    }
                    Err(e) => {
                        tx.rollback().expect("transaction rollback failed");
                        error!("Outbound connection Failure {:#?}", e);
                        dead_node_keys.insert(old_id);
                        result = Err(e);
                    }
                }
            }
            Node::InboundConnectionNode(mut node) => {
                info!("Mapping sesion id for InboundConnectionNode");
                let old_id = node.clone_key();
                let session_id = cache.check_cache(&node)?;
                if let Some(new_node_key) = session_id {
                    node.set_key(new_node_key.clone());
                    key_map.insert(old_id, new_node_key.clone());
                    output_subgraph.nodes.insert(new_node_key, node.into());
                    continue
                }

                let mut tx = conn.start_transaction(
                    false,
                    Some(IsolationLevel::Serializable),
                    Some(false)
                ).expect("Failed to acquire transaction");

                let attribution_res = attribute_session_node(&mut tx, &node, should_default);

                match attribution_res {
                    Ok(new_node_key) => {
                        info!("Successfully attributed session for InboundConnectionNode");
                        tx.commit().expect("transaction commit failed");
                        node.set_key(new_node_key.clone());
                        key_map.insert(old_id, new_node_key.clone());
                        output_subgraph.nodes.insert(new_node_key.clone(), node.clone().into());
                        cache.update_cache(&node, new_node_key);
                    }
                    Err(e) => {
                        tx.rollback().expect("transaction rollback failed");
                        error!("Inbound Connection Failure {:#?}", e);
                        dead_node_keys.insert(old_id);
                        result = Err(e);
                    }
                }
            }
            Node::IpAddressNode(mut node) => {
                key_map.insert(node.clone_key(), node.clone_key());
                output_subgraph.nodes.insert(node.clone_key(), node.into());
            }
        }
    }

    result
}

