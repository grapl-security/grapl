use mysql::{Pool, Transaction};

use failure::Error;
use graph_descriptions::graph_description::*;
use graph_descriptions::graph_description::host::*;
use graph_descriptions::*;


use uuid;

pub fn get_ip_asset_session_id(conn: &mut Transaction,
                          ip: &str,
                          timestamp: u64) -> Result<Option<String>, Error> {
    info!("get_ip_asset_session_id");
    let query = format!(r"
       SELECT session_id, create_time
       FROM ip_asset_history
       WHERE ip = {}
             AND create_time <= {}
       ORDER BY create_time DESC",
            &ip,
            &timestamp
    );

    let query_result = conn.prep_exec(&query,
                                  &())?;

    for row in query_result {
        let row = row.unwrap();
        let a_time: i64 = row.get("create_time").unwrap();
        let a_time: u64 = a_time as u64;

        if timestamp > a_time {
            return Ok(Some(row.get("session_id").unwrap()));
        }
    }

    Ok(None)
}


pub fn create_ip_asset_session(conn: &Pool,
                          ip: &str,
                          session_id: &str,
                          create_time: u64) -> Result<(), Error> {
    info!("create_ip_asset_session");

    let query = format!(r"
       INSERT INTO ip_asset_history
          (session_id, ip, create_time)
          VALUES
              ({}, {}, {})",
            &session_id, ip, &create_time
    );

    conn.prep_exec(&query, &(
        &session_id,
        &ip,
        &create_time.to_string()
    ))?;
    Ok(())
}

pub fn create_table(conn: &Pool) {
    info!("Creating ip_asset_history table");
//    conn.prep_exec("DROP TABLE IF EXISTS `ip_asset_history`", &());

    conn.prep_exec("CREATE TABLE IF NOT EXISTS ip_asset_history (
                primary_key     SERIAL PRIMARY KEY,
                session_id      TEXT NOT NULL,
                ip              TEXT NOT NULL,
                create_time     NUMERIC NOT NULL
              )", &()).expect("ip_asset_history::create_table");
}


pub fn attribute_ip_asset_process_node(conn: &mut Transaction,
                  node: &mut ProcessDescriptionProto) -> Result<(), Error> {
    info!("attribute_ip_asset_process_node");

    if let HostId::Ip(ref ip) = node.host_id.as_ref().unwrap().host_id.as_ref().unwrap() {
        let asset_id = match ProcessState::from(node.state) {
            ProcessState::Created => {
                get_ip_asset_session_id(
                    conn,ip,node.timestamp
                )?.unwrap()
            },
            ProcessState::Existing => {
                get_ip_asset_session_id(
                    conn,ip,node.timestamp
                )?.unwrap()
            },
            ProcessState::Terminated => {
                get_ip_asset_session_id(
                    conn,ip,node.timestamp
                )?.unwrap()
            },
        };

        let asset_id = HostIdentifier::AssetId(asset_id).into();

        node.host_id = Some(asset_id);
        return Ok(());
    }

    info!("{:#?}", node.host_id);
    Ok(())

//    unimplemented!()
}

pub fn attribute_ip_asset_file_node(conn: &mut Transaction,
                                       node: &mut FileDescriptionProto) -> Result<(), Error> {
    info!("attribute_ip_asset_file_node");

    if let HostId::Ip(ref ip) = node.host_id.as_ref().unwrap().host_id.as_ref().unwrap() {
        let asset_id = match FileState::from(node.state) {
            FileState::Created => get_ip_asset_session_id(
                conn,ip,node.timestamp
            )?.unwrap(),
            FileState::Existing => get_ip_asset_session_id(
                conn,ip,node.timestamp
            )?.unwrap(),
            FileState::Deleted => get_ip_asset_session_id(
                    conn,ip,node.timestamp
                )?.unwrap(),
        };

        let asset_id = HostIdentifier::AssetId(asset_id).into();

        node.host_id = Some(asset_id);
        return Ok(());
    }

    info!("{:#?}", node.host_id);
    Ok(())


//    unimplemented!()
}

pub fn map_asset_ids_to_graph(conn: &mut Transaction,
                              subgraph: &mut GraphDescription) -> Result<(), Error> {

    info!("map_asset_ids_to_graph");

    for _node in subgraph.nodes.clone() {
        let node: NodeDescription = _node.1.into();
        match node.which() {
            Node::ProcessNode(mut node) => {
                attribute_ip_asset_process_node(conn, &mut node)?;
                subgraph.nodes.insert(_node.0, node.into());
            }
            Node::FileNode(mut node) => {
                attribute_ip_asset_file_node(conn, &mut node)?;
                subgraph.nodes.insert(_node.0, node.into());
            }
            _ => continue
        }
    }

    Ok(())
}
