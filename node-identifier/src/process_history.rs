use mysql::Pool;

use failure::Error;
use graph_descriptions::graph_description::*;
use graph_descriptions::*;

use std::collections::HashMap;

use uuid;
use std::collections::HashSet;

pub fn get_process_session_id(conn: &Pool,
                          pid: u64,
                          asset_id: &str,
                          timestamp: u64) -> Result<Option<String>, Error> {
    info!("get process session id");
    let query = format!("
       SELECT session_id, create_time
       FROM process_history
       WHERE pid = {} AND asset_id = \"{}\"
             AND create_time <= {}
       ORDER BY create_time DESC",
        pid, asset_id, timestamp
    );

    info!("Query is: {}", &query);

    let query_results = conn.prep_exec(&query, &())?;

    info!("get_process_session_id prep_exec");

    let query_results: Vec<_> = query_results.collect();

    for row in &query_results {
        info!("Row {:#?}", row);
        let row = row.as_ref().expect("Failed to unwrap row");
        let a_time: i64 = row.get("create_time").expect("create_time");
        let a_time: u64 = a_time as u64;

        if timestamp > a_time {
            return Ok(Some(row.get("session_id").expect("create_time")));
        }
    }

    if !query_results.is_empty() {
        info!("Retrieving session id for latest process");
        let row = query_results.last().unwrap();
        let row = row.as_ref().expect("Failed to unwrap row");

        return Ok(Some(row.get("session_id").expect("session_id")));
    }

    info!("Went through all query results");

    Ok(None)
}

pub fn check_exact_process(conn: &Pool,
                           pid: u64,
                           asset_id: &str,
                           create_time: u64) -> Result<Option<String>, Error> {

    // TODO: We can probably add a bit of skew here, +/- 5 seconds would be safe
    let query = format!("
       SELECT session_id
       FROM process_history
       WHERE pid = {} AND asset_id = \"{}\"
             AND create_time = {}
       ORDER BY create_time DESC",
                        pid, asset_id, create_time
    );

    let query_result = conn.prep_exec(&query, &())?;

    let row = query_result.into_iter().next();

    if let Some(row) = row {
        return Ok(Some(row?.get("session_id").expect("session_id")));
    }

    Ok(None)
}

pub fn create_process_session(conn: &Pool,
                          pid: u64,
                          asset_id: &str,
                          create_time: u64) -> Result<String, Error> {
    info!("create process session id");

    // Check if we've already processed a process start with these exact values
    let maybe_id = check_exact_process(conn, pid, asset_id, create_time)?;

    if let Some(session_id) = maybe_id {
        return Ok(session_id)
    }

    let session_id = uuid::Uuid::new_v4().to_string();

    let query = format!("
       INSERT INTO process_history
          (session_id, pid, asset_id, create_time)
          VALUES
              (\"{}\", {}, \"{}\", {})",
        session_id, pid, asset_id, create_time
    );

    info!("create_process_session prep_exec {}", &query);
    conn.prep_exec(&query, &())?;

    Ok(session_id)
}


pub fn update_or_create(conn: &Pool,
                    pid: u64,
                    asset_id: &str,
                    create_time: u64,
                    should_default: bool
) -> Result<String, Error> {
    info!("update or create process session id");

    let session_id = get_process_session_id(
        conn, pid, asset_id, create_time
    )?;

    if let Some(session_id) = session_id {
        info!("Got process session_id");
        return Ok(session_id)
    }

    if should_default {
        info!("Did not get session id. Creating process session_id");
        create_process_session(conn, pid, asset_id, create_time)
    } else {
        bail!("Failed to get the process session id, did not default.")
    }
}

pub fn create_table(conn: &Pool) {
    info!("Creating process_history  table");
//    conn.prep_exec("DROP TABLE IF EXISTS `process_history`", &());

    conn.prep_exec("CREATE TABLE IF NOT EXISTS process_history (
                    primary_key     SERIAL PRIMARY KEY,
                    session_id      TEXT NOT NULL,
                    asset_id        TEXT NOT NULL,
                    pid             NUMERIC NOT NULL,
                    create_time     NUMERIC NOT NULL
                  )", &()).expect("process_history::create_table");
}


pub fn attribute_process_node(conn: &Pool,
                              node: &mut ProcessDescriptionProto,
                              should_default: bool
) -> Result<(), Error> {

    let session_id = match ProcessState::from(node.state) {
        ProcessState::Created => {
            info!("Handling created process");

            create_process_session(
                &conn, node.pid, node.asset_id(), node.timestamp
            )?
        },
        ProcessState::Existing => {
            info!("Handling existing process");
            update_or_create(
                &conn, node.pid, node.asset_id(), node.timestamp,
                should_default
            )?
        },
        ProcessState::Terminated => {
            warn!("Unimplemented!: Handling terminated process {:#?}", node);
//            let session_id = get_process_session_id(
//                &conn, node.pid, node.asset_id(), node.timestamp
//            )?;
//
            unimplemented!()
        },
    };

    node.node_key = session_id;
    Ok(())
}

pub fn remap_edges(key_map: &HashMap<String, String>,
                   dead_node_keys: &HashSet<String>,
                   subgraph: &mut GraphDescription) {
    let mut edge_map = HashMap::new();
    for (node_key, edges) in subgraph.edges.clone() {
        let edges = edges.edges;
        let mut new_edges = Vec::with_capacity(edges.len());

        for edge in edges {
            let from_neighbor_key = key_map.get(&edge.from_neighbor_key)
                .unwrap_or(&edge.from_neighbor_key).to_owned();
            let to_neighbor_key = key_map.get(&edge.to_neighbor_key)
                .unwrap_or(&edge.to_neighbor_key).to_owned();

            if dead_node_keys.contains(&from_neighbor_key) {
                continue
            }

            if dead_node_keys.contains(&to_neighbor_key) {
                continue
            }

            let new_edge = EdgeDescriptionProto {
                from_neighbor_key,
                to_neighbor_key,
                edge_name: edge.edge_name
            };

            new_edges.push(new_edge)
        }

        let node_key = key_map.get(&node_key)
            .unwrap_or(&node_key).clone();

        edge_map.insert(node_key,
                        EdgeListProto {edges: new_edges});
    }
    subgraph.edges = edge_map;
}


pub fn map_process_session_ids_to_graph(conn: &Pool,
                                        subgraph: &mut GraphDescription,
                                        should_default: bool
) -> Result<(), Error> {

    // Maps old session ids to new ones
    let mut key_map = HashMap::new();
    let mut dead_node_keys = HashSet::new();

    let mut result = Ok(());

    for _node in subgraph.nodes.clone() {
        let node: NodeDescription = _node.1.into();
        if let Node::ProcessNode(mut node) = node.which() {
            let attribution_res = attribute_process_node(&conn, &mut node, should_default);

            if let e @ Err(_) = attribution_res {
                subgraph.nodes.remove(&_node.0);
                dead_node_keys.insert(_node.0);
                result = e;
                continue
            }

            key_map.insert(_node.0.clone(), node.node_key.clone());

            // Replace old node with new node
            subgraph.nodes.remove(&_node.0);
            subgraph.nodes.insert(node.node_key.clone(), node.into());

        }
    }

    remap_edges(&key_map, &dead_node_keys, subgraph);

    result
}

