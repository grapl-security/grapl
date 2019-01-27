use mysql::{Pool, Transaction};

use failure::Error;
use graph_descriptions::graph_description::*;
use graph_descriptions::graph_description::host::*;
use graph_descriptions::*;


use uuid;
use std::collections::HashSet;
use std::collections::HashMap;
use mysql::IsolationLevel;
use std::str;

pub fn get_ip_asset_asset_id(conn: &mut Transaction,
                          ip: &[u8],
                          timestamp: u64) -> Result<Option<String>, Error> {
    info!("get_ip_asset_asset_id");
    let query = format!(r#"
       SELECT asset_id, create_time
       FROM ip_asset_history
       WHERE ip = "{}"
             AND create_time <= {}
       ORDER BY create_time DESC"#,
       str::from_utf8(ip).expect("ip is not valid utf8"), &timestamp
    );

    let query_result = conn.prep_exec(
        &query,
        &()
    )?;

    for row in query_result {
        let row = row.expect("row failed");
        let a_time: u64 = row.get("create_time").expect("create_time");

        if timestamp >= a_time {
            return Ok(Some(row.get("asset_id").expect("asset_id")));
        }
    }

    Ok(None)
}

pub fn get_hostname_asset_asset_id(conn: &mut Transaction,
                             hostname: &str,
                             timestamp: u64) -> Result<Option<String>, Error> {
    info!("get_ip_asset_asset_id");
    let query = format!(r#"
       SELECT asset_id, create_time
       FROM hostname_asset_history
       WHERE hostname = "{}"
             AND create_time <= {}
       ORDER BY create_time DESC"#,
                        hostname, &timestamp
    );

    let query_result = conn.prep_exec(
        &query,
        &()
    )?;

    for row in query_result {
        let row = row.expect("row failed");
        let a_time: u64 = row.get("create_time").expect("create_time");

        if timestamp >= a_time {
            return Ok(Some(row.get("asset_id").expect("asset_id")));
        }
    }

    Ok(None)
}



pub fn create_table(conn: &Pool) {
    info!("Creating ip_asset_history table");
//    conn.prep_exec("DROP TABLE IF EXISTS `ip_asset_history`", &());

    conn.prep_exec("CREATE TABLE IF NOT EXISTS ip_asset_history (
                primary_key     SERIAL PRIMARY KEY,
                asset_id        TEXT NOT NULL,
                ip              BLOB NOT NULL,
                create_time     BIGINT UNSIGNED NOT NULL
              )", &()).expect("ip_asset_history::create_table");
}


pub fn attribute_asset(conn: &mut Transaction, host_id: &HostId, timestamp: u64) -> Result<String, Error> {
    info!("attribute_ip_asset_process_node");

    match host_id {
        HostId::Ip(ref ip) => {
            let asset_id = get_ip_asset_asset_id(
                conn,ip, timestamp
            )?.expect(&format!("Failed to retrieve asset id from ip address {:?}", str::from_utf8(ip)));

            Ok(asset_id)
        }
        HostId::AssetId(asset_id) => {
            Ok(asset_id.to_owned())
        }
        HostId::Hostname(ref hostname) => {
            let asset_id = get_hostname_asset_asset_id(
                conn,hostname, timestamp
            )?.expect(&format!("Failed to retrieve asset id from hostnameaddress {:?}", hostname));

            Ok(asset_id)
        }
    }

}


pub fn map_asset_ids_to_graph(conn: &Pool,
                              dead_node_keys: &mut HashSet<String>,
                              unid_subgraph: &mut GraphDescription,
) -> Result<(), Error> {

    info!("map_asset_ids_to_graph");
    let mut result = Ok(());

    for _node in unid_subgraph.nodes.clone() {
        let node: NodeDescription = _node.1.into();
        match node.which() {
            Node::ProcessNode(mut node) => {
                let old_id = node.clone_key().to_owned();

                let mut tx = conn.start_transaction(
                    false,
                    Some(IsolationLevel::Serializable),
                    Some(true)
                ).expect("Failed to acquire transaction");


                let host_id = match (&node.asset_id, &node.hostname, &node.host_ip) {
                    (Some(asset_id), _, _) => HostId::AssetId(asset_id.clone()),
                    (_, Some(hostname), _) => HostId::Hostname(hostname.clone()),
                    (_, _, Some(host_ip)) => HostId::Ip(host_ip.clone()),
                    (_, _, _) => panic!("Must provide at least one of: asset_id, hostname, host_ip")
                };

                let attribution_res = attribute_asset(&mut tx, &host_id, node.timestamp());

                match attribution_res {
                    Ok(new_asset_id) => {
                        tx.commit().expect("transaction commit failed");
                        node.set_asset_id(new_asset_id.clone());
                        unid_subgraph.nodes.insert(node.clone_key(), node.into());
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
                let old_id = node.clone_key().to_owned();

                let mut tx = conn.start_transaction(
                    false,
                    Some(IsolationLevel::Serializable),
                    Some(true)
                ).expect("Failed to acquire transaction");

                let host_id = match (&node.asset_id, &node.hostname, &node.host_ip) {
                    (Some(asset_id), _, _) => HostId::AssetId(asset_id.clone()),
                    (_, Some(hostname), _) => HostId::Hostname(hostname.clone()),
                    (_, _, Some(host_ip)) => HostId::Ip(host_ip.clone()),
                    (_, _, _) => panic!("Must provide at least one of: asset_id, hostname, host_ip")
                };

                let attribution_res = attribute_asset(&mut tx, &host_id, node.timestamp());

                match attribution_res {
                    Ok(new_asset_id) => {
                        tx.commit().expect("transaction commit failed");
                        node.set_asset_id(new_asset_id.clone());
//                        node.set_key(new_node_key.clone());
                        unid_subgraph.nodes.insert(node.clone_key(), node.into());
                    }
                    Err(e) => {
                        tx.rollback().expect("transaction rollback failed");
                        error!("File Attribution Failure {:#?}", e);
                        dead_node_keys.insert(old_id);
                        result = Err(e);
                    }
                }

            }
            Node::InboundConnectionNode(mut node) => {
                let old_id = node.clone_key().to_owned();

                let mut tx = conn.start_transaction(
                    false,
                    Some(IsolationLevel::Serializable),
                    Some(true)
                ).expect("Failed to acquire transaction");

                let host_id = match (&node.asset_id, &node.hostname, &node.host_ip) {
                    (Some(asset_id), _, _) => HostId::AssetId(asset_id.clone()),
                    (_, Some(hostname), _) => HostId::Hostname(hostname.clone()),
                    (_, _, Some(host_ip)) => HostId::Ip(host_ip.clone()),
                    (_, _, _) => panic!("Must provide at least one of: asset_id, hostname, host_ip")
                };


                let attribution_res = attribute_asset(&mut tx, &host_id, node.timestamp());

                match attribution_res {
                    Ok(new_asset_id) => {
                        tx.commit().expect("transaction commit failed");
                        node.set_asset_id(new_asset_id.clone());
//                        node.set_key(new_node_key.clone());
                        unid_subgraph.nodes.insert(node.clone_key(), node.into());

                    }
                    Err(e) => {
                        tx.rollback().expect("transaction rollback failed");
                        error!("InboundConnection Attribution Failure {:#?}", e);
                        dead_node_keys.insert(old_id);
                        result = Err(e);
                    }
                }

            }
            Node::OutboundConnectionNode(mut node) => {
                let old_id = node.clone_key().to_owned();

                let mut tx = conn.start_transaction(
                    false,
                    Some(IsolationLevel::Serializable),
                    Some(true)
                ).expect("Failed to acquire transaction");

                let host_id = match (&node.asset_id, &node.hostname, &node.host_ip) {
                    (Some(asset_id), _, _) => HostId::AssetId(asset_id.clone()),
                    (_, Some(hostname), _) => HostId::Hostname(hostname.clone()),
                    (_, _, Some(host_ip)) => HostId::Ip(host_ip.clone()),
                    (_, _, _) => panic!("Must provide at least one of: asset_id, hostname, host_ip")
                };

                let attribution_res = attribute_asset(&mut tx, &host_id, node.timestamp());

                match attribution_res {
                    Ok(new_asset_id) => {
                        tx.commit().expect("transaction commit failed");
                        node.set_asset_id(new_asset_id.clone());
                        unid_subgraph.nodes.insert(node.clone_key(), node.into());

                    }
                    Err(e) => {
                        tx.rollback().expect("transaction rollback failed");
                        error!("Outbound Connection Attribution Failure {:#?}", e);
                        dead_node_keys.insert(old_id);
                        result = Err(e);
                    }
                }

            }
            _ => continue
        }
    }

    result
}


