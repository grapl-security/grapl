use std::{
    cell::RefCell,
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    rc::Rc,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
        Mutex,
    },
};

use async_recursion::async_recursion;
use futures::{
    future::{
        self,
        join_all,
        try_join_all,
    },
    TryFutureExt,
};
use itertools::Itertools;
use scylla::{
    cql_to_rust::FromCqlVal,
    FromUserType,
    IntoTypedRows,
    IntoUserType,
    Session,
};

use crate::{
    graph_view::Graph,
    node_query::InnerNodeQuery,
    node_view::Node,
    visited::Visited,
};

#[derive(Debug, Clone)]
pub struct StringField {
    pub uid: Uid,
    pub populated_field: PropertyName,
    pub value: String,
}

// We should not return a Node but instead a Graph
// And we'll then mark which node in the graph corresponds with the root

#[derive(Debug, Clone)]
pub struct EdgeRow {
    pub source_uid: Uid,
    pub f_edge_name: EdgeName,
    pub r_edge_name: EdgeName,
    pub destination_uid: Uid,
    pub tenant_id: uuid::Uuid,
}

pub use rust_proto_new::graplinc::grapl::api::graph_query::v1beta1::messages::StringCmp;
use rust_proto_new::graplinc::grapl::common::v1beta1::types::{
    EdgeName,
    NodeType,
    PropertyName,
    Uid,
};

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{
            self,
            BufRead,
        },
        path::Path,
    };

    use maplit::hashmap;
    use scylla::{
        batch::Consistency,
        SessionBuilder,
    };

    use super::*;
    use crate::node_query::NodeQuery;

    async fn insert_string_ix(
        session: Arc<Session>,
        tenant_id: &uuid::Uuid,
        node_type: &str,
        uid: i64,
        populated_field: String,
        value: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        session
            .query(
                r#"
                    INSERT INTO immutable_string_index (node_type, uid, populated_field, value)
                      VALUES (?, ?, ?, ?)"#,
                (node_type, uid, populated_field, value),
            )
            .await?;

        Ok(())
    }

    async fn insert_edge(
        session: Arc<Session>,
        tenant_id: &uuid::Uuid,
        source_uid: i64,
        f_edge_name: String,
        r_edge_name: String,
        destination_uid: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        session
            .query(
                r#"
            INSERT INTO edge (
                source_uid,
                f_edge_name,
                r_edge_name,
                destination_uid
            )
            VALUES (?, ?, ?, ?)"#,
                (
                    source_uid.clone(),
                    f_edge_name.clone(),
                    r_edge_name.clone(),
                    destination_uid.clone(),
                ),
            )
            .await?;

        session
            .query(
                r#"
            INSERT INTO edge (
                source_uid,
                f_edge_name,
                r_edge_name,
                destination_uid
            )
            VALUES (?, ?, ?, ?)"#,
                (destination_uid, r_edge_name, f_edge_name, source_uid),
            )
            .await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn smoke_test() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let uris = &["localhost"][..];
        println!("connecting to {uris:?}");
        let session: Session = SessionBuilder::new()
            .known_nodes(&uris[..])
            .default_consistency(Consistency::One)
            //     .user(
            //     "scylla", "cS0h4mLIWxaEB5D",
            // )
            .build()
            .await?;
        let session = Arc::new(session);

        session.query("DROP KEYSPACE if exists ix", &[]).await?;
        session.query("CREATE KEYSPACE IF NOT EXISTS ix WITH REPLICATION = {'class' : 'SimpleStrategy', 'replication_factor' : 3}", &[]).await?;
        session.query("USE ix", &[]).await?;
        session
            .query("DROP TABLE IF EXISTS immutable_string_index", &[])
            .await?;
        // return Ok(());
        println!("created keyspace");

        session
            .query(
                "
                CREATE TABLE IF NOT EXISTS immutable_string_index (
                       uid bigint,
                       node_type text,
                       populated_field text,
                       value text,
                       PRIMARY KEY ((uid, populated_field), node_type)
                )
                WITH compression = {
                    'sstable_compression': 'LZ4Compressor',
                    'chunk_length_in_kb': 64
                };
                ",
                &[],
            )
            .await?;

        println!("created imm");

        session
            .query(
                "
                CREATE TABLE IF NOT EXISTS edge (
                       source_uid bigint,
                       f_edge_name text,
                       r_edge_name text,
                       destination_uid bigint,
                       PRIMARY KEY((source_uid, f_edge_name, destination_uid))
                )
                WITH compression = {
                    'sstable_compression': 'LZ4Compressor',
                    'chunk_length_in_kb': 64
                };
                ",
                &[],
            )
            .await?;
        println!("Created edge");
        let uid = 1000;
        let tenant_id = uuid::Uuid::parse_str("c6d7cf37-911c-454d-83f9-763bc03c2e44")?;

        // let res = session.query("SELECT count(uid) FROM immutable_string_index USING TIMEOUT 120s", &[]).await;
        // dbg!(&res);

        let uid = 1000;
        let tenant_id = uuid::Uuid::parse_str("c6d7cf37-911c-454d-83f9-763bc03c2e44")?;
        insert_string_ix(
            session.clone(),
            &tenant_id,
            "Process",
            uid,
            "process_name".to_string(),
            "chrome.exe".to_string(),
        )
        .await?;
        println!("inserted string ix");

        insert_string_ix(
            session.clone(),
            &tenant_id,
            "Process",
            uid,
            "arguments".to_string(),
            "-a -f -b --boop=bop".to_string(),
        )
        .await?;

        insert_string_ix(
            session.clone(),
            &tenant_id,
            "Process",
            uid + 123,
            "process_name".to_string(),
            "evil.exe".to_string(),
        )
        .await?;

        insert_edge(
            session.clone(),
            &tenant_id,
            uid,
            "children".into(),
            "parent".into(),
            uid + 123,
        )
        .await?;

        let query = NodeQuery::root("Process".try_into()?)
            .with_string_comparisons(
                "process_name".try_into()?,
                [StringCmp::eq("chrome.exe", false)],
            )
            .with_edge_to(
                "children".try_into()?,
                "parent".try_into()?,
                "Process".try_into()?,
                |neighbor| {
                    neighbor.with_string_comparisons(
                        "process_name".try_into().expect("invalid name"),
                        [StringCmp::eq("chrome.exe", true)],
                    );
                },
            )
            .build();

        let response = query
            .query_graph(Uid::from_i64(uid + 123).unwrap(), tenant_id, session)
            .await?;
        if let Some(ref graph) = response {
            println!("node_count: {}", graph.get_nodes().len());
            for node in graph.get_nodes() {
                println!("node: {:?}", node);
            }
            let root_node = graph.find_node_by_query_id(query.root_query_id).unwrap();
            println!("----response----\n{root_node:#?}");

            for (edge_name, neighbor) in graph.edges.iter() {
                println!("edge_name: {:?} neighbor: {:?}", edge_name, neighbor);
            }
        } else {
            println!("no response");
        }
        Ok(())
    }

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}

#[derive(Clone, Default)]
pub struct GraphQuery {
    pub root_query_id: u64,
    pub nodes: HashMap<u64, InnerNodeQuery>,
    pub edges: HashMap<(u64, EdgeName), HashSet<u64>>,
    pub edge_map: HashMap<EdgeName, EdgeName>,
}

impl GraphQuery {
    pub fn add_node(&mut self, query_id: u64, node_type: NodeType) {
        self.nodes.insert(
            query_id,
            InnerNodeQuery {
                query_id,
                node_type,
                string_filters: HashMap::new(),
            },
        );
    }

    #[tracing::instrument(skip(self, session))]
    pub async fn query_graph(
        &self,
        uid: Uid,
        tenant_id: uuid::Uuid,
        session: Arc<Session>,
    ) -> Result<Option<Graph>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut query_handles = Vec::with_capacity(self.nodes.len());
        // We should add a way for different queries to short circuit each other
        for node_query in self.nodes.values() {
            let session = session.clone();
            let node_query = node_query.clone();
            query_handles.push(async move {
                let visited = Visited::new();
                node_query
                    .fetch_node_with_edges(&self, uid, tenant_id, session, visited)
                    .await
            });
        }
        for graph in try_join_all(query_handles).await? {
            if let Some(graph) = graph {
                return Ok(Some(graph));
            }
        }
        Ok(None)
    }
}
