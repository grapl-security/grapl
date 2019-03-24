extern crate aws_lambda_events;
extern crate base58;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;
extern crate lru_time_cache;
#[macro_use]
extern crate mysql;
extern crate prost;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate sha2;
extern crate sqs_lambda;
extern crate stopwatch;
extern crate uuid;
extern crate zstd;
extern crate simple_logger;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate serde;

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use base58::ToBase58;
use failure::Error;
use futures::future::Future;
use graph_descriptions::*;
use graph_descriptions::graph_description::*;
use lambda::Context;
use lambda::error::HandlerError;
use lru_time_cache::LruCache;
use mysql as my;
use prost::Message;
use rusoto_core::Region;
use rusoto_s3::{S3, S3Client};
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};
use sha2::{Digest, Sha256};
use sqs_lambda::NopSqsCompletionHandler;
use sqs_lambda::EventHandler;
use sqs_lambda::events_from_s3_sns_sqs;
use sqs_lambda::S3EventRetriever;
use sqs_lambda::SqsService;
use sqs_lambda::ZstdProtoDecoder;
use stopwatch::Stopwatch;

use cache::IdentityCache;
use ip_asset_history::map_asset_ids_to_graph;
use session_history::map_session_ids_to_graph;

macro_rules! log_time {
    ($msg:expr, $x:expr) => {
        {
            let mut sw = Stopwatch::start_new();
            #[allow(path_statements)]
            let result = $x;
            sw.stop();
            info!("{} {} milliseconds", $msg, sw.elapsed_ms());
            result
        }
    };
}

pub mod ip_asset_history;
pub mod session_history;
pub mod cache;
pub mod session;

#[derive(Clone)]
struct NodeIdentifier<'a, F>
    where F: (Fn(GraphDescription) -> Result<(), Error>) + Clone
{
    lru_cache: IdentityCache<'a>,
    should_default: bool,
    address: String,
    db_name: String,
    output_handler: F
}

impl<'a, F> NodeIdentifier<'a, F>
    where F: (Fn(GraphDescription) -> Result<(), Error>) + Clone
{
    pub fn new(
        address: impl Into<String>,
        db_name: impl Into<String>,
        lru_cache: IdentityCache<'a>,
        output_handler: F,
        should_default: bool,
    ) -> Self {
        Self {
            address: address.into(),
            db_name: db_name.into(),
            lru_cache,
            should_default,
            output_handler,
        }
    }
}


impl<'a, F> EventHandler<GeneratedSubgraphs> for NodeIdentifier<'a, F>
    where F: (Fn(GraphDescription) -> Result<(), Error>) + Clone
{
    fn handle_event(&self, event: GeneratedSubgraphs) -> Result<(), Error> {
        let mut subgraphs = event;
        info!("Handling raw event");

        if subgraphs.subgraphs.is_empty() {
            return Ok(())
        }

        info!("Connecting to history database");

        let username = env::var("HISTORY_DB_USERNAME")?;
        let password = env::var("HISTORY_DB_PASSWORD")?;

        let pool = my::Pool::new(
            format!("mysql://{username}:{password}@{address}:3306/{db_name}",
                    username=username,
                    address=self.address,
                    password=password,
                    db_name=self.db_name
            )
        )?;

        info!("Connected to history database");

        info!("Handling {} subgraphs", subgraphs.subgraphs.len());

        log_time!{
            "creating tables",
            {
                ip_asset_history::create_table(&pool);
//                hostname_asset_history::create_table(&pool);
                session_history::create_process_table(&pool);
                session_history::create_file_table(&pool);
                session_history::create_connection_table(&pool);
            }
        }

        subgraphs.subgraphs.sort_unstable_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let mut total_subgraph = GraphDescription::new(subgraphs.subgraphs[0].timestamp);

        let mut result = Ok(());
        for unid_subgraph in subgraphs.subgraphs {
            let lru_cache = self.lru_cache.clone();
            let _result: Result<(), Error> = (|| {
                let mut output_subgraph = GraphDescription::new(unid_subgraph.timestamp);
                let mut unid_subgraph: GraphDescription = unid_subgraph.into();
                let mut result = Ok(());

                let mut unid_id_map = HashMap::new();
                let mut dead_node_ids = HashSet::new();

                info!("Mapping asset ids to graph");

                let r = map_asset_ids_to_graph(
                    &pool,
                    &mut dead_node_ids,
                    &mut unid_subgraph,
                );
                if let e @ Err(_) = r {
                    error!("error: {:#?}", e);
                    result = e;
                }

                info!("removing {} nodes and their edges", dead_node_ids.len());
                remove_dead_nodes(&dead_node_ids, &mut unid_subgraph);
                dead_node_ids.clear();

                info!("Mapping process session ids to graph");

                // Process/ File mapping *must* happen after asset ids
                let r = map_session_ids_to_graph(
                    &pool,
                    &mut unid_id_map,
                    &mut dead_node_ids,
                    &unid_subgraph,
                    &mut output_subgraph,
                    self.should_default,
                    lru_cache
                );

                if let e @ Err(_) = r {
                    error!("error: {:#?}", e);
                    result = e;
                }


                log_time! {
                    "remap_edges",
                    remap_edges(&unid_id_map, &dead_node_ids, &unid_subgraph, &mut output_subgraph)
                }

                total_subgraph.merge(&output_subgraph);

                result
            })();

            if let e @ Err(_) = _result {
                error!("error: {:#?}", e);
                result = e;
            }
        }

        if !total_subgraph.is_empty() {
            (self.output_handler)(total_subgraph)?;
        }

        result
    }
}

pub fn handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    let max_count = 100_000;
    let time_to_live = Duration::from_secs(60 * 5);

    let username = env::var("HISTORY_DB_USERNAME").expect("HISTORY_DB_USERNAME");
    let mysql_addr = env::var("HISTORY_DB_ADDRESS").expect("HISTORY_DB_ADDRESS");
    let lru_cache = IdentityCache::new(max_count, time_to_live, b"pepper");

    let handler = NodeIdentifier::new(
        mysql_addr,
        "historydb",
        lru_cache,
        upload_identified_graphs,
        false
    );

    let region = Region::UsEast1;
//    info!("Creating sqs_client");
//    let sqs_client = Arc::new(SqsClient::simple(region.clone()));

    info!("Creating s3_client");
    let s3_client = Arc::new(S3Client::simple(region.clone()));

    info!("Creating retriever");
    let retriever = S3EventRetriever::new(
        s3_client,
        |d| {info!("Parsing: {:?}", d); events_from_s3_sns_sqs(d)},
        ZstdProtoDecoder{},
    );

    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    info!("Creating sqs_completion_handler");
    let sqs_completion_handler = NopSqsCompletionHandler::new(
        queue_url
    );

    let mut sqs_service = SqsService::new(
        retriever,
        handler,
        sqs_completion_handler,
    );

    info!("Handing off event");
    sqs_service.run(event, ctx)?;

    Ok(())
}

pub fn retry_handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    let max_count = 100_000;
    let time_to_live = Duration::from_secs(60 * 5);

    let username = env::var("HISTORY_DB_USERNAME").expect("IDENTITY_CACHE_PEPPER");
    let mysql_addr = env::var("HISTORY_DB_ADDRESS").expect("HISTORY_DB_ADDRESS");

    let lru_cache = IdentityCache::new(max_count, time_to_live, b"pepper");

    let handler = NodeIdentifier::new(
        mysql_addr,
        "historydb",
        lru_cache,
        upload_identified_graphs,
        true
    );

    let region = Region::UsEast1;
//    info!("Creating sqs_client");
//    let sqs_client = Arc::new(SqsClient::simple(region.clone()));

    info!("Creating s3_client");
    let s3_client = Arc::new(S3Client::simple(region.clone()));

    info!("Creating retriever");
    let retriever = S3EventRetriever::new(
        s3_client,
        |d| {info!("Parsing: {:?}", d); events_from_s3_sns_sqs(d)},
        ZstdProtoDecoder{},
    );

    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    info!("Creating sqs_completion_handler");
    let sqs_completion_handler = NopSqsCompletionHandler::new(
        queue_url
    );

    let mut sqs_service = SqsService::new(
        retriever,
        handler,
        sqs_completion_handler,
    );

    info!("Handing off event");
    sqs_service.run(event, ctx)?;

    Ok(())
}



pub fn remove_dead_nodes(dead_node_ids: &HashSet<String>,
                         unid_subgraph: &mut GraphDescription) {
    for node_id in dead_node_ids.iter() {
        unid_subgraph.nodes.remove(node_id);
    }

    for (_node_key, edges) in unid_subgraph.edges.iter_mut() {
        let mut new_edges = vec![];
        for edge in &edges.edges {
            if dead_node_ids.contains(&edge.from_neighbor_key) ||
                dead_node_ids.contains(&edge.to_neighbor_key) {
                continue
            }
            new_edges.push(edge.clone());
        }
        edges.edges = new_edges;
    }

}



pub fn remap_edges(key_map: &HashMap<String, String>,
                    dead_node_ids: &HashSet<String>,
                    input_subgraph: &GraphDescription,
                    output_subgraph: &mut GraphDescription) {

    for (_node_key, edges) in &input_subgraph.edges {
        for edge in &edges.edges {

            if dead_node_ids.contains(&edge.from_neighbor_key) {
                warn!("Removing edge.from_neighbor_key {}", edge.from_neighbor_key);
                continue
            }

            if dead_node_ids.contains(&edge.to_neighbor_key) {
                warn!("Removing edge.to_neighbor_key {}", edge.to_neighbor_key);
                continue
            }

            let from_neighbor_key = key_map.get(&edge.from_neighbor_key)
                .expect("from_neighbor_key");
            let to_neighbor_key = key_map.get(&edge.to_neighbor_key)
                .expect("to_neighbor_key");

            output_subgraph.add_edge(
                edge.edge_name.to_owned(),
                from_neighbor_key.to_owned(),
                to_neighbor_key.to_owned(),
            )
        }
    }
}

pub fn upload_identified_graphs(subgraph: GraphDescription) -> Result<(), Error> {
    info!("Uploading identified subgraphs");
    let s3 = S3Client::simple(
        Region::UsEast1
    );

    let subgraph: GraphDescription = subgraph.into();

    let mut body = Vec::with_capacity(5000);
    subgraph.encode(&mut body).expect("Failed to encode subgraph");

    let mut compressed = Vec::with_capacity(body.len());
    let mut proto = Cursor::new(&body);

    zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
        .expect("compress zstd capnp");

    let mut hasher = Sha256::default();
    hasher.input(&body);

    let key = hasher.result().as_ref().to_base58();

    let bucket_prefix = std::env::var("BUCKET_PREFIX").expect("BUCKET_PREFIX");

    let bucket = bucket_prefix + "-subgraphs-generated-bucket";
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH).unwrap().as_secs();

    let day = epoch - (epoch % (24 * 60 * 60));

    let key = format!("{}/{}",
                      day,
                      key
    );
    info!("Uploading identified subgraphs to {}", key);
    s3.put_object(
        &rusoto_s3::PutObjectRequest {
            bucket,
            key: key.clone(),
            body: Some(compressed),
            ..Default::default()
        }
    ).wait()?;
    info!("Uploaded identified subgraphs to {}", key);

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use session_history::check_exact_session;
    use mysql::IsolationLevel;
    use std::sync::mpsc::channel;

    fn get_handler(address: impl Into<String>, should_default: bool) -> NodeIdentifier<'static, impl (Fn(GraphDescription) -> Result<(), Error>) + Clone>
    {

        let max_count = 100_000;
        let time_to_live = Duration::from_secs(60 * 5);

        let lru_cache = IdentityCache::new(max_count, time_to_live, b"pepper");

        let handler = NodeIdentifier::new(
            "localhost",
            address,
            lru_cache,
            move |graph_description| {
                info!("graph descriptions {:?}", graph_description);
                Ok(())
            },
            should_default
        );

        handler
    }

    fn get_handler_with(address: impl Into<String>, should_default: bool, output: impl (Fn(GraphDescription) -> Result<(), Error>) + Clone) -> NodeIdentifier<'static, impl (Fn(GraphDescription) -> Result<(), Error>) + Clone>
    {

        let max_count = 100_000;
        let time_to_live = Duration::from_secs(60 * 5);

        let lru_cache = IdentityCache::new(max_count, time_to_live, b"pepper");

        let handler = NodeIdentifier::new(
            "localhost",
            address,
            lru_cache,
            output,
            should_default
        );

        handler
    }

    fn rebuild_process_table(pool: &my::Pool) {
        pool.prep_exec("DROP TABLE IF EXISTS `process_history`", &()).expect("Failed to drop table");

        session_history::create_process_table(&pool);
    }

    fn create_table_if_not_exists(table_name: impl AsRef<str>) {

        {
            let pool = my::Pool::new(
                "mysql://root:secret@localhost:3306/"
            ).expect("pool");

            let mut conn = pool.get_conn().unwrap();
            conn.query(format!("CREATE DATABASE IF NOT EXISTS {};", table_name.as_ref())).expect("create db");
        }

    }

    fn ms_since_epoch() -> u64 {
        let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let event_time = since_the_epoch.as_secs() as u128 * 1000 +
            since_the_epoch.subsec_millis() as u128;
        let event_time = event_time as u64;
        event_time
    }

    fn set_env_vars() {
        env::set_var("BUCKET_PREFIX", "unique_id");
        env::set_var("HISTORY_DB_USERNAME", "root");
        env::set_var("HISTORY_DB_PASSWORD", "secret");
    }


    // TODO: Assert that the values in the database match up appropriately
    #[test]
    fn test_process_create() {
        simple_logger::init();
        set_env_vars();

        create_table_if_not_exists("test_process_create");

        let pool = my::Pool::new(
            "mysql://root:secret@localhost:3306/test_process_create"
        ).expect("pool");

        rebuild_process_table(&pool);

        let event_time = ms_since_epoch();

        let mut graph = GraphDescription::new(event_time);

        let process = ProcessDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(ProcessState::Created)
            .pid(123u64)
            .created_timestamp(event_time)
            .build()
            .unwrap();

        graph.add_node(process.clone());

        let unid_subgraphs = GeneratedSubgraphs::new(vec![
            graph
        ]);

        debug!("Handling event");

        let handler = get_handler("test_process_create", false);

        handler.handle_event(unid_subgraphs).expect("handle_event failed");

        let mut tx = pool.start_transaction(
            false,
            Some(IsolationLevel::Serializable),
            Some(true)
        ).expect("Failed to acquire transaction");


        let session_check = check_exact_session(
            &mut tx,
            &&process
        )
            .expect("Session check failed 1")
            .expect("Session check failed 2");

        assert!(!session_check.is_empty());
        assert_ne!(session_check, process.node_key);
    }

    // TODO: Assert that the values in the database match up appropriately
    #[test]
    fn test_file_create() {
        simple_logger::init();
        set_env_vars();

        const test_name: &'static str = "test_file_create";

        create_table_if_not_exists(test_name);

        let pool = my::Pool::new(
            "mysql://root:secret@localhost:3306/test_file_create"
        ).expect("pool");

        rebuild_process_table(&pool);

        let event_time = ms_since_epoch();

        let mut graph = GraphDescription::new(event_time);

        let process = FileDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(FileState::Created)
            .path(r"C:\Users\andy\AppData\Local\Programs\Python\Python37\Lib\genericpath.py")
            .created_timestamp(event_time)
            .build()
            .unwrap();

        graph.add_node(process.clone());

        let unid_subgraphs = GeneratedSubgraphs::new(vec![
            graph
        ]);

        debug!("Handling event");

        let handler = get_handler(test_name, false);

        handler.handle_event(unid_subgraphs).expect("handle_event failed");

        let mut tx = pool.start_transaction(
            false,
            Some(IsolationLevel::Serializable),
            Some(true)
        ).expect("Failed to acquire transaction");


        let session_check = check_exact_session(
            &mut tx,
            &&process
        )
            .expect("Session check failed 1")
            .expect("Session check failed 2");

        assert!(!session_check.is_empty());
        assert_ne!(session_check, process.node_key);
    }



    #[test]
    fn test_process_last_seen_no_default() {
        simple_logger::init();

        set_env_vars();

        create_table_if_not_exists("test_process_last_seen_no_default");

        let pool = my::Pool::new(
            "mysql://root:secret@localhost:3306/test_process_last_seen_no_default"
        ).expect("pool");

        rebuild_process_table(&pool);

        let event_time = ms_since_epoch();

        let mut graph = GraphDescription::new(event_time);

        let process = ProcessDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(ProcessState::Existing)
            .pid(123u64)
            .last_seen_timestamp(event_time)
            .build()
            .unwrap();

        graph.add_node(process.clone());

        let unid_subgraphs = GeneratedSubgraphs::new(vec![
            graph
        ]);

        debug!("Handling event");

        let handler = get_handler("test_process_last_seen_no_default", false);

        let handle_result = handler.handle_event(unid_subgraphs);
        assert!(handle_result.is_err()); // TODO: Be more specific

        let mut tx = pool.start_transaction(
            false,
            Some(IsolationLevel::Serializable),
            Some(true)
        ).expect("Failed to acquire transaction");


        let session_check = check_exact_session(
            &mut tx,
            &&process
        )
            .expect("Session check failed 1");
        // We should not see this session in our db
        assert!(session_check.is_none());
    }

    #[test]
    fn test_process_last_seen_with_default() {
        simple_logger::init();

        set_env_vars();

        create_table_if_not_exists("test_process_last_seen_with_default");

        let pool = my::Pool::new(
            "mysql://root:secret@localhost:3306/test_process_last_seen_with_default"
        ).expect("pool");

        rebuild_process_table(&pool);

        let event_time = ms_since_epoch();

        let mut graph = GraphDescription::new(event_time);

        let process_a_seen_first = ProcessDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(ProcessState::Existing)
            .pid(123u64)
            .last_seen_timestamp(event_time)
            .build()
            .unwrap();

        let process_a_seen_second = ProcessDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(ProcessState::Existing)
            .pid(123u64)
            .last_seen_timestamp(event_time + 100_000) // Long enough to be out of cache
            .build()
            .unwrap();

        graph.add_node(process_a_seen_first.clone());
        graph.add_node(process_a_seen_second.clone());

        let unid_subgraphs = GeneratedSubgraphs::new(vec![
            graph
        ]);

        let handler = get_handler("test_process_last_seen_with_default", true);

        let handle_result = handler.handle_event(unid_subgraphs).expect("Handle event failed");

        let mut tx = pool.start_transaction(
            true,
            Some(IsolationLevel::Serializable),
            Some(true)
        ).expect("Failed to acquire transaction");

        let session_check_first = check_exact_session(
            &mut tx,
            &&process_a_seen_first
        )
            .expect("Session check first failed 1");

        let session_check_second = check_exact_session(
            &mut tx,
            &&process_a_seen_second
        )
            .expect("Session check first failed 1");

        dbg!(session_check_first.clone());
        dbg!(session_check_second.clone());

        // There's no need to guarantee which gets processed first
        assert!(session_check_first.is_some() || session_check_second.is_some());

        let res = pool.prep_exec("SELECT * from process_history", &()).expect("Failed to drop table");
        let query_results: Vec<_> = res.collect();

        assert_eq!(query_results.len(), 1);
        let row = &query_results[0].as_ref().expect("Failed to get row");
        let session_id: String = row.get("session_id").expect("session_id");
//        assert_eq!(session_id, session_check_first, "session_id, session_check_first");
    }

    /// ### Testcase: Process Guessing with last seen times
    /// This test is to ensure that the "guessing" behavior for sessions is correct, even
    /// when there are no definitive, non-guessed session IDs in the table.
    ///
    /// Guessing occurs when we have are referencing a node without the necessary information
    /// to derive its node_key. For example, if we see a process read a file, we don't know the
    /// create time of that process, and therefor must try to derive its ID.
    ///
    /// Guessing must handle the case where the table only contains guesses, and ensure that
    /// the guesses "propagate" ie: we don't guess a new ID.
    ///
    /// This special case only occurs when a table is empty, or otherwise has no non-guess IDs, but
    /// for most session types that's actually the common case.
    ///
    /// ### Given
    /// an empty process_history table
    ///
    /// ### When
    /// A graph (A) with one Existing process is attributed
    /// A graph (B) with one Existing process is attributed
    /// The Existing processes (A) and (B) are at different times
    ///
    /// ### Then
    /// There should be a single entry in the database for both (A) and (B)
    /// The graphs (A) and (B) will each have one process with the same node_key
    #[test]
    fn test_process_guessing_with_last_seen() {
        simple_logger::init();

        set_env_vars();

        create_table_if_not_exists("test_process_guessing_with_last_seen");

        let (tx, rx) = channel();

        let pool = my::Pool::new(
            "mysql://root:secret@localhost:3306/test_process_guessing_with_last_seen"
        ).expect("pool");

        rebuild_process_table(&pool);
        let event_time = ms_since_epoch();

        let mut graph = GraphDescription::new(event_time );

        let process_a_seen_first = ProcessDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(ProcessState::Existing)
            .pid(123u64)
            .last_seen_timestamp(event_time)
            .build()
            .unwrap();

        graph.add_node(process_a_seen_first.clone());

        let unid_subgraphs = GeneratedSubgraphs::new(vec![
            graph
        ]);

        debug!("Handling event");

        let handler =
            get_handler_with(
                "test_process_guessing_with_last_seen",
                true,
                move |identified_graph| {
                    tx.send(identified_graph).expect("send identified_graph");
                    Ok(())
                }
            );

        handler.handle_event(unid_subgraphs).expect("Handle event failed");

        let mut graph = GraphDescription::new(12345);

        let process_a_seen_second = ProcessDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(ProcessState::Existing)
            .pid(123u64)
            .last_seen_timestamp(event_time)
            .build()
            .unwrap();

        graph.add_node(process_a_seen_second.clone());

        let unid_subgraphs = GeneratedSubgraphs::new(vec![
            graph
        ]);

        handler.handle_event(unid_subgraphs).expect("Handle event failed");

        let query_results = pool.prep_exec("SELECT * from process_history", &()).expect("Failed to drop table");
        let query_results: Vec<_> = query_results.collect();

        let graph_a = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("graph_a");

        let graph_b = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("graph_b");

        let process_a_key = graph_a.nodes.values().next().unwrap().get_key();
        let process_b_key = graph_b.nodes.values().next().unwrap().get_key();

        assert_eq!(process_a_key, process_b_key, "keys should be the same");
        // Assert that the attributed graphs match what we sent in - they should both have the
        // same uuid for the process with pid 123
        assert_eq!(query_results.len(), 1);
        let row = &query_results[0].as_ref().expect("Failde to get row");
        let session_id: String = row.get("session_id").expect("session_id");
    }

    /// ### Testcase: Process Creation after Guess
    /// This test is to ensure that, in the event of a Guess at a timestamp A, and a Creation event
    /// at timestamp B where A > B, if the Guess is processed first the Creation will use the ID of
    /// the Guess.
    ///
    /// This case will occur when events are processed out of order. The defaulting behavior will
    /// create the guess, but a Create event may come in after.
    ///
    /// ### Given
    /// an empty process_history table
    ///
    /// ### When
    /// A graph (A) with one Existing process is attributed
    /// A graph (B) with one Created process is attributed
    /// (A)'s process is at timestamp 'a
    /// (B)'s process is at timestamp 'b
    /// 'a > 'b
    ///
    /// ### Then
    /// There should be two entries in the database, one for the Existing and one for the Creation
    /// The two entries should contain the same ID

    #[test]
    fn test_process_create_after_guess() {
        simple_logger::init();

        set_env_vars();

        create_table_if_not_exists("test_process_create_after_guess");

        let (tx, rx) = channel();

        let pool = my::Pool::new(
            "mysql://root:secret@localhost:3306/test_process_create_after_guess"
        ).expect("pool");

        rebuild_process_table(&pool);
        let event_time = ms_since_epoch();

        let mut guess_graph = GraphDescription::new(event_time);
        let mut create_graph = GraphDescription::new(event_time);

        let process_to_guess = ProcessDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(ProcessState::Existing)
            .pid(123u64)
            .last_seen_timestamp(event_time + 100_000)
            .build()
            .unwrap();

        let process_to_create = ProcessDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(ProcessState::Created)
            .pid(123u64)
            .created_timestamp(event_time)
            .build()
            .unwrap();

        guess_graph.add_node(process_to_guess);
        create_graph.add_node(process_to_create);

        let subgraphs_with_guess = GeneratedSubgraphs::new(vec![
            guess_graph
        ]);


        let subgraphs_with_create = GeneratedSubgraphs::new(vec![
            create_graph
        ]);

        let handler =
            get_handler_with(
                "test_process_create_after_guess",
                true,
                move |identified_graph| {
                    tx.send(identified_graph).expect("send identified_graph");
                    Ok(())
                }
            );

        handler.handle_event(subgraphs_with_guess).expect("Handle guess event failed");
        handler.handle_event(subgraphs_with_create).expect("Handle create event failed");

        let query_results = pool.prep_exec("SELECT * from process_history", &()).expect("Failed to drop table");
        let query_results: Vec<_> = dbg!(query_results.collect());

        let graph_a = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("graph_a");

        let graph_b = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("graph_b");

        let process_a_key = graph_a.nodes.values().next().unwrap().get_key();
        let process_b_key = graph_b.nodes.values().next().unwrap().get_key();

        assert_eq!(process_a_key, process_b_key, "keys should be the same");
        // Assert that the attributed graphs match what we sent in - they should both have the
        // same uuid for the process with pid 123
        assert_eq!(query_results.len(), 2);
        let row1 = &query_results[0].as_ref().expect("Failed to get row");
        let row2 = &query_results[1].as_ref().expect("Failed to get row");
        let session_id1: String = row1.get("session_id").expect("session_id");
        let session_id2: String = row2.get("session_id").expect("session_id");
        assert_eq!(session_id1, session_id2, "keys should be the same");
    }

    /// ### Testcase: Process Creation Replay
    /// This test is to ensure that, when a Creation event is replayed (this may happen due to timeouts
    /// or errors), only a single ID across the events.
    ///
    /// ### Given
    /// an empty process_history table
    ///
    /// ### When
    /// A graph (A) with one Created process is attributed
    /// A graph (B) with one Created process is attributed, the process is identical to graph (A)'s
    ///
    /// ### Then
    /// The database should contain one value for the process
    /// The attributed processes should both have the same ID's
    #[test]
    fn test_process_create_replay() {
        let test_name = "test_process_create_replay";

        let event_time = ms_since_epoch();

        simple_logger::init();
        set_env_vars();

        create_table_if_not_exists(test_name);

        let pool = my::Pool::new(
            format!("mysql://root:secret@localhost:3306/{}", test_name)
        ).expect("pool");

        rebuild_process_table(&pool);

        let mut graph_a = GraphDescription::new(event_time);
        let mut graph_b = GraphDescription::new(event_time);

        let process = ProcessDescriptionBuilder::default()
            .asset_id("asset_id".to_owned())
            .state(ProcessState::Created)
            .pid(123u64)
            .created_timestamp(event_time)
            .build()
            .unwrap();

        graph_a.add_node(process.clone());
        graph_b.add_node(process.clone());

        let unid_subgraphs_a = GeneratedSubgraphs::new(vec![
            graph_a
        ]);

        let unid_subgraphs_b = GeneratedSubgraphs::new(vec![
            graph_b
        ]);

        debug!("Handling event");

        let (tx, rx) = channel();

        let handler =
            get_handler_with(
                test_name,
                false,
                move |identified_graph| {
                    tx.send(identified_graph).expect("send identified_graph");
                    Ok(())
                }
            );

        handler.handle_event(unid_subgraphs_a).expect("handle_event failed");
        handler.handle_event(unid_subgraphs_b).expect("handle_event failed");

        let mut tx = pool.start_transaction(
            false,
            Some(IsolationLevel::Serializable),
            Some(true)
        ).expect("Failed to acquire transaction");


        let session_check = check_exact_session(
            &mut tx,
            &&process
        )
            .expect("Session check failed 1")
            .expect("Session check failed 2");

        assert!(!session_check.is_empty());
        assert_ne!(session_check, process.node_key);

        // Check that both attributed processes have the same ID
        let process_a_key = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("graph_a");

        let process_b_key = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("graph_b");

        let process_a_key = process_a_key.nodes.values().next().unwrap().get_key();
        let process_b_key = process_b_key.nodes.values().next().unwrap().get_key();

        assert_eq!(process_a_key, process_b_key);

        // Assert that only one ID was created

        let query_results = pool.prep_exec("SELECT * from process_history", &()).expect("Failed to drop table");
        let query_results: Vec<_> = dbg!(query_results.collect());
        assert_eq!(query_results.len(), 1);
    }

}