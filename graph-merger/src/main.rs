extern crate rand;
extern crate aws_lambda_events;
extern crate base64;
extern crate base16;
extern crate dgraph_rs;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate grpc;
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;
extern crate openssl_probe;
extern crate prost;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sns;
extern crate rusoto_sqs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate sha2;
extern crate simple_logger;
extern crate sqs_lambda;
extern crate stopwatch;
extern crate itertools;

use std::str::FromStr;
use rand::thread_rng;
use rand::seq::SliceRandom;

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use futures::Future;
use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use dgraph_rs::protos::api;
use dgraph_rs::protos::api_grpc::{Dgraph, DgraphClient};
use dgraph_rs::protos::api_grpc;
use dgraph_rs::Transaction;
use failure::Error;
use graph_descriptions::graph_description::*;
use grpc::{Client, ClientStub};
use grpc::ClientConf;
use lambda::Context;
use lambda::error::HandlerError;
use lambda::lambda;
use prost::Message;
use rusoto_core::Region;
use rusoto_s3::{S3, S3Client};
use rusoto_sns::{Sns, SnsClient};
use rusoto_sns::PublishInput;
use rusoto_sqs::{GetQueueUrlRequest, Sqs, SqsClient};
use sha2::{Digest, Sha256};
use sqs_lambda::EventHandler;
use sqs_lambda::events_from_s3_sns_sqs;
use sqs_lambda::NopSqsCompletionHandler;
use sqs_lambda::S3EventRetriever;
use sqs_lambda::SqsService;
use sqs_lambda::ZstdProtoDecoder;
use itertools::Itertools;

macro_rules! log_time {
    ($msg:expr, $x:expr) => {
        {
            let mut sw = stopwatch::Stopwatch::start_new();
            #[allow(path_statements)]
            let result = $x;
            sw.stop();
            info!("{} {} milliseconds", $msg, sw.elapsed_ms());
            result
        }
    };
}


#[derive(Debug, Fail)]
enum MergeFailure {
    #[fail(display = "Transaction failure")]
    TransactionFailure
}

#[derive(Serialize, Deserialize, Debug)]
struct Uid {
    uid: String
}

#[derive(Serialize, Deserialize, Debug)]
struct DgraphResponse {
    response: HashMap<String, Vec<Uid>>,
}

struct DgraphQuery<'a> {
    node_key: &'a str,
    s_key: String,
    query: String,
    insert: serde_json::Value,
}

impl<'a> DgraphQuery<'a> {

    fn get_skey(&self) -> &str {
        &self.s_key
    }

    fn mut_insert(&mut self) -> &mut serde_json::Value {
        &mut self.insert
    }


    fn get_insert(&mut self) -> & serde_json::Value {
        &self.insert
    }
}


impl<'a> From<(usize, &'a NodeDescription)> for DgraphQuery<'a> {
    fn from((key, node): (usize, &'a NodeDescription)) -> DgraphQuery<'a> {
        let key = key as u16;
        let mut s_key = String::from("q");
        s_key.push_str(&key.to_string());

        let node_key = node.get_key();
        let query = format!(r#"
            {s_key}(func: eq(node_key, "{node_key}"))
            {{
                uid,
            }}
        "#, s_key=s_key, node_key=node_key);

        let mut insert = (*node).clone().into_json();

        for value in insert.as_object_mut().unwrap().values_mut() {
            if let Some(s_value) = value.clone().as_str() {
                let escaped_value = s_value.replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                    .replace("\'", "\\\'")
                    .replace("\n", "\\\n")
                    .replace("\t", "\\\t");

                *value = serde_json::Value::from(escaped_value);
            }
        }

        DgraphQuery {
            node_key,
            s_key,
            query,
            insert
        }
    }
}


struct BulkUpserter<'a> {
    queries: Vec<DgraphQuery<'a>>,
    client: &'a DgraphClient,
    query_buffer: String,
    insert_buffer: String,
    node_key_to_uid: HashMap<String, String>
}

impl<'a> BulkUpserter<'a> {
    pub fn new(client: &'a DgraphClient, nodes: impl IntoIterator<Item=&'a NodeDescription>) -> BulkUpserter<'a> {
        let nodes = nodes.into_iter();
        let nodes_len = nodes.size_hint();
        let queries: Vec<_> = nodes.enumerate().map(DgraphQuery::from).collect();

        let buf_len: usize = queries.iter().map(|q| &q.query).map(String::len).sum();
        let buf_len = buf_len + queries.len();

        let query_buffer= String::with_capacity(buf_len + 3);
        let insert_buffer= String::with_capacity(buf_len + 3);

        BulkUpserter {
            queries,
            client,
            query_buffer,
            insert_buffer,
            node_key_to_uid: HashMap::with_capacity(nodes_len.1.unwrap_or(nodes_len.0))
        }
    }

    pub fn upsert_all(&mut self) -> Result<(), Error> {
        let mut txn = Transaction::new(&self.client);

        info!("Generating queries");
        // clear, and then fill, the internal query buffer
        log_time!("generate_queries", self.generate_queries());

        info!("Querying all nodes");
        // Query dgraph for remaining nodes
        let query_response =
            log_time!("query_all", self.query_all(&mut txn)?);

        info!("Generating inserts");
        // Generate upserts
        log_time!("generate_insert", self.generate_insert(query_response)?);

        info!("Performing bulk upsert");
        // perform upsert
        let mut mutation = api::Mutation::new();

        mutation.set_json = self.insert_buffer.as_bytes().to_owned();

        let mut_res = log_time!("txn.mutate", txn.mutate(mutation)?);

        let txn_commit = log_time!("txn.commit", txn.commit()?);

        // We need to take the newly created uids and add them to our map
        self.node_key_to_uid.extend(mut_res.uids);

        Ok(())
    }

    fn generate_queries(&mut self) {
        self.query_buffer.clear();
        let all_queries = &mut self.query_buffer;

        all_queries.push_str("{");
        let joined = self.queries.iter().map(|query| &query.query).join(",");
        all_queries.push_str(&joined);
        all_queries.push_str("}");

    }

    fn query_all(&mut self, txn: &mut Transaction) -> Result<DgraphResponse, Error> {
        let resp = txn.query(
            &self.query_buffer[..]
        );

        let resp = match resp {
            Ok(resp) => resp,
            Err(e) => {
                error!("Query failed with {}. Buffer: {:?}", e, self.query_buffer);
                bail!(e);
            }
        };

        Ok(DgraphResponse{response: serde_json::from_slice(&resp.json)?})
    }

    fn generate_insert(&mut self, response: DgraphResponse) -> Result<(), Error> {
        self.insert_buffer.clear();
        let insert_buffer = &mut self.insert_buffer;
        insert_buffer.push_str("[");
        for to_insert in &mut self.queries {
            let response = response.response.get(to_insert.get_skey());

            match response.map(Vec::as_slice) {
                Some([uid]) => {
                    self.node_key_to_uid.insert(to_insert.node_key.into(), uid.uid.clone());
                    to_insert.mut_insert()["uid"] = serde_json::Value::from(uid.uid.clone());

                },
                // If we get an empty response we just create the node
                Some([]) => {
                    let placeholder = format!("_:{}", to_insert.node_key);

                    to_insert.mut_insert()["uid"] = serde_json::Value::from(placeholder);
                },
                // We should never get more than a single uid back
                Some(uids) if uids.len() > 1 => bail!("Got more than one response"),
                // If we generate a query we should never *not* have it in a response
                None => bail!("Could not find response"),
                _ => unreachable!("until slice patterns improve")

            };

            let insert = &to_insert.get_insert().to_string();

            insert_buffer.push_str(insert);
            insert_buffer.push_str(",");
        }

        info!("popped {:#?}", insert_buffer.pop());
        if insert_buffer.is_empty() {
            bail!("insert_buffer empty");
        }
        insert_buffer.push_str("]");

        Ok(())
    }
}

fn insert_edges<'a>(client: &DgraphClient,
                edges: impl IntoIterator<Item=&'a EdgeDescription>,
                key_uid: &HashMap<String, String>) -> Result<(), Error> {

    if key_uid.is_empty() {
        warn!("key_uid is empty");
        return Ok(())
    }

    let bulk_insert = generate_bulk_edge_insert(edges, key_uid)?;

    let mut mutation = api::Mutation::new();
    mutation.commit_now = true;
    mutation.set_json = bulk_insert.into_bytes();


    info!("insert_edges {:?}", client.mutate(Default::default(), mutation).wait()?);

    Ok(())
}

fn generate_bulk_edge_insert<'a>(edges: impl IntoIterator<Item=&'a EdgeDescription>,
                             key_uid: &HashMap<String, String>) -> Result<String, Error> {

    if key_uid.is_empty() {
        bail!("key_uid must not be empty");
    }

    let edges = edges.into_iter();
    let edges_len = edges.size_hint();
    let edges_len = edges_len.1.unwrap_or(edges_len.0);
    let mut bulk_insert = String::with_capacity(48 * edges_len);

    bulk_insert.push_str("[");
    for edge in edges {
        let to = &key_uid
            .get(edge.to_neighbor_key.as_str());
        let from = &key_uid
            .get(edge.from_neighbor_key.as_str());

        // TODO: Add better logs, with actual identifiers
        let (to, from) = match (to, from) {
            (None, None) => {
                warn!("Failed to map node_key to uid for to and from: {:?}", edge);
                continue
            },
            (_, None) => {
                warn!("Failed to map node_key to uid for from: {:?}", edge);
                continue
            },
            (None, _) => {
                warn!("Failed to map node_key to uid for to: {:?}", edge);
                continue
            },
            (Some(to), Some(from)) => (to, from),
        };

        let insert_statement = generate_edge_insert(&to, &from, &edge.edge_name);
        bulk_insert.push_str(&insert_statement);
        bulk_insert.push_str(",");
    }
    // Eat the trailing comma, replace with "]"
    bulk_insert.pop();
    if bulk_insert.is_empty() {
        bail!("Failed to generate any edge insertions")
    }
    bulk_insert.push_str("]");
    Ok(bulk_insert)
}

fn generate_edge_insert(to: &str, from: &str, edge_name: &str) -> String {
    json!({
        "uid": from,
        edge_name: {
            "uid": to
        }
    }).to_string()
}



fn with_retries<T>(mut f: impl FnMut() -> Result<T, Error>) -> Result<T, Error> {

    let max = 6;
    let mut cur = 0;
    loop {
        match f() {
            t @ Ok(_) => break t,
            Err(e) => {
                if cur == max {
                    return Err(e)
                } else {
                    error!("with_retries: {}", e);
                    cur += 1;
                    std::thread::sleep_ms(cur * 25);
                }
            }

        }
    }
}

pub fn subgraph_to_sns<S>(sns_client: &S, subgraphs: &GraphDescription) -> Result<(), Error>
    where S: Sns
{
    // TODO: Preallocate buffers
    info!("upload_subgraphs");
    let mut proto = Vec::with_capacity(5000);
    subgraphs.encode(&mut proto)?;

    let subgraph_merged_topic_arn = std::env::var("SUBGRAPH_MERGED_TOPIC_ARN").expect("SUBGRAPH_MERGED_TOPIC_ARN");

    let mut compressed = Vec::with_capacity(proto.len());
    let mut proto = Cursor::new(&proto);
    zstd::stream::copy_encode(&mut proto, &mut compressed, 4)
        .expect("compress zstd capnp");

    let message = base64::encode(&compressed);

    info!("Message is {} bytes", message.len());

    sns_client.publish(
        PublishInput {
            message,
            topic_arn: subgraph_merged_topic_arn.into(),
            ..Default::default()
        }
    )
        .with_timeout(Duration::from_secs(5))
        .sync()?;

    Ok(())
}


#[derive(Clone)]
struct GraphMerger {
    mg_alphas: Vec<String>,
}


impl EventHandler<GraphDescription> for GraphMerger {
    fn handle_event(&self, subgraph: GraphDescription) -> Result<(), Error> {
        println!("handling new subgraph with {} nodes", subgraph.nodes.len());

        if subgraph.is_empty() {
            warn!("Attempted to merge empty subgraph. Short circuiting.");
            return Ok(())
        }

        let mut rng = thread_rng();
        let rand_alpha = self.mg_alphas.choose(&mut rng)
            .expect("Empty rand_alpha");

        let mg_client = &api_grpc::DgraphClient::with_client(
            Arc::new(
                Client::new_plain(rand_alpha, 9080, ClientConf {
                    ..Default::default()
                })?
            )
        );

        let mut upserter = BulkUpserter::new(
            &mg_client,
            subgraph.nodes.values()
        );

        // Even if some node upserts fail we should create edges for the ones that succeeded
        let upsert_res = with_retries(|| {
            log_time!("upsert_all", upserter.upsert_all())
        });

        if upserter.node_key_to_uid.is_empty() {
            bail!("Failed to attribute uids to any of {} node keys", subgraph.nodes.len());
        }

        let edge_res = with_retries(|| {
            let edges: Vec<_> = subgraph.edges.values().map(|e| &e.edges).flatten().collect();
            if edges.is_empty() {
                return Ok(())
            }

            info!("inserting {} edges", edges.len());

            log_time!("insert_edges", insert_edges(&mg_client, edges, &upserter.node_key_to_uid))
        });

        // If our node_key_to_uid map isn't empty we must have merged at least a single node,
        // so even if all edges failed, or even if some upserts failed, we should output the graph
        let region = {
            let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
            Region::from_str(&region_str).expect("Invalid Region")
        };

        let sns_client = SnsClient::new(region);
        if let Err(e) = subgraph_to_sns(&sns_client, &subgraph) {
            bail!("subgraph sns publish failed: {}", e)
        };

        upsert_res?;
        edge_res?;

        Ok(())
    }
}

pub fn handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");

    let mg_alphas: Vec<_> = std::env::var("MG_ALPHAS").expect("MG_ALPHAS")
        .split(',')
        .map(str::to_string)
        .collect();

    info!("Connecting to alphas: {:?}", mg_alphas);

    let handler = GraphMerger{
        mg_alphas
    };
    let region: Region = {
        let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
        Region::from_str(&region_str).expect("Invalid Region")
    };

//    info!("Creating sqs_client");
//    let sqs_client = Arc::new(SqsClient::new(region.clone()));

    info!("Creating s3_client");
    let s3_client = Arc::new(S3Client::new(region.clone()));

    info!("Creating retriever");
    let retriever = S3EventRetriever::new(
        s3_client,
        |d| {info!("Parsing: {:?}", d); events_from_s3_sns_sqs(d)},
        ZstdProtoDecoder{},
    );



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
    log_time!("sqs_service.run", sqs_service.run(event, ctx)?);

    Ok(())
}


fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    openssl_probe::init_ssl_cert_env_vars();

    lambda!(handler);
}

pub fn set_process_schema(client: &DgraphClient) {
    let mut op_schema = api::Operation::new();
    op_schema.drop_all = false;
    op_schema.schema = concat!(
       		"node_key: string @upsert @index(hash) .\n",
       		"pid: int @index(int) .\n",
       		"created_time: int @index(int) .\n",
       		"asset_id: string @index(hash) .\n",
       		"terminate_time: int @index(int) .\n",
       		"image_name: string @index(hash) @index(fulltext) .\n",
       		"arguments: string  @index(fulltext) .\n",
       		"bin_file: uid @reverse .\n",
       		"children: uid @reverse .\n",
       		"created_files: uid @reverse .\n",
            "deleted_files: uid @reverse .\n",
            "read_files: uid @reverse .\n",
            "wrote_files: uid @reverse .\n",
            "created_connection: uid @reverse .\n",
            "bound_connection: uid @reverse .\n",
        ).to_string();
    client.alter(Default::default(), op_schema).wait().expect("set schema");
}

pub fn set_file_schema(client: &DgraphClient) {
    let mut op_schema = api::Operation::new();
    op_schema.drop_all = false;
    op_schema.schema = r#"
       		node_key: string @upsert @index(hash) .
       		asset_id: string @index(hash) .
       		create_time: int @index(int) .
       		delete_time: int @index(int) .
       		path: string @index(hash) @index(fulltext) .
        "#.to_string();
    client.alter(Default::default(), op_schema).wait().expect("set schema");
}

pub fn set_ip_address_schema(client: &DgraphClient) {
    let mut op_schema = api::Operation::new();
    op_schema.drop_all = false;
    op_schema.schema = r#"
       		node_key: string @upsert @index(hash) .
       		last_seen: int @index(int) .
       		external_ip: string @index(hash) .
        "#.to_string();
    client.alter(Default::default(), op_schema).wait().expect("set schema");
}

pub fn set_connection_schema(client: &DgraphClient) {
    let mut op_schema = api::Operation::new();
    op_schema.drop_all = false;
    op_schema.schema = concat!(
       		"node_key: string @upsert @index(hash) .\n",
       		"create_time: int @index(int) .\n",
       		"terminate_time: int @index(int) .\n",
       		"last_seen_time: int @index(int) .\n",
       		"ip: string @index(hash) .\n",
       		"port: string @index(hash) .\n",
       		// outbound connections have a `connection` edge to inbound connections
       		"connection: uid @reverse .\n",
       		// outbound connections have a `connection` edge to external ip addresses
       		"external_connection: uid @reverse .\n",
    ).to_string();
    client.alter(Default::default(), op_schema).wait().expect("set schema");
}
