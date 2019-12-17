extern crate aws_lambda_events;
extern crate base16;
extern crate base64;
extern crate dgraph_rs;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate graph_descriptions;
extern crate grpc;
extern crate itertools;
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;
extern crate openssl_probe;
extern crate prost;
extern crate rand;
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

use std::collections::HashMap;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use dgraph_rs::DgraphClient;
use dgraph_rs::protos::api;
use dgraph_rs::protos::api_grpc;
use failure::Error;
use futures::Future;
use futures::future::join_all;

use graph_descriptions::node::NodeT;
use graph_descriptions::graph_description::{Graph, Node};
use graph_descriptions::graph_description::node::WhichNode;
use graph_descriptions::process::ProcessState;
use graph_descriptions::file::FileState;
use graph_descriptions::process_inbound_connection::ProcessInboundConnectionState;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;
use graph_descriptions::network_connection::NetworkConnectionState;

use grpc::{Client, ClientStub};
use grpc::ClientConf;
use itertools::Itertools;
use lambda::Context;
use lambda::error::HandlerError;
use lambda::lambda;
use prost::Message;
use rand::seq::SliceRandom;
use rand::thread_rng;
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

use crate::futures::FutureExt;
use serde_json::Value;
use std::iter::FromIterator;

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

fn generate_edge_insert(from: &str, to: &str, edge_name: &str) -> api::Mutation {
    let mu = json!({
        "uid": from,
        edge_name: {
            "uid": to
        }
    }).to_string().into_bytes();

    let mut mutation = api::Mutation::new();
    mutation.commit_now = true;
    mutation.set_json = mu;
    mutation
}

async fn node_key_to_uid(dg: &DgraphClient, node_key: &str) -> Result<Option<String>, Error> {

    let mut txn = dg.new_read_only();

    const QUERY: & str = r"
       query q0($a: string)
    {
        q0(func: eq(node_key, $a), first: 1) {
            uid
        }
    }
    ";

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), node_key.into());

    let query_res: Value = txn.query_with_vars(QUERY, vars).await
        .map(|res| serde_json::from_slice(&res.json))??;

    let uid = query_res.get("q0")
        .and_then(|res| res.get(0))
        .and_then(|uid| uid.get("uid"))
        .and_then(|uid| uid.as_str())
        .map(String::from);

    Ok(uid)
}

async fn upsert_node(dg: &DgraphClient, node: Node) -> Result<String, Error> {
    let query = format!(r#"
                {{
                  p as var(func: eq(node_key, "{}"), first: 1)
                }}
                "#, node.get_node_key());

    let node_key = node.clone_node_key();
    let mut set_json = node.into_json();
    set_json["uid"] = "uid(p)".into();


    let mu = api::Mutation {
        set_json: set_json.to_string().into_bytes(),
        commit_now: true,
        ..Default::default()
    };

    let mut txn = dg.new_txn();
    let upsert_res = txn.upsert(
        query, mu,
    )
        .await
        .expect("Request to dgraph failed");

    txn.commit_or_abort().await?;

    info!("Upsert res: {:?}", upsert_res);

    if let Some(uid) = upsert_res.uids.values().next() {
        Ok(uid.to_owned())
    } else {
        match node_key_to_uid(dg, &node_key).await? {
            Some(uid) => {
                Ok(uid)
            },
            None => bail!("Could not retrieve uid after upsert"),
        }
    }
}


fn chunk<T, U>(data: U, count: usize) -> Vec<U>
    where U: IntoIterator<Item=T>,
          U: FromIterator<T>,
          <U as IntoIterator>::IntoIter: ExactSizeIterator
{

    let mut iter = data.into_iter();
    let iter = iter.by_ref();

    let chunk_len = (iter.len() / count) as usize + 1;

    let mut chunks = Vec::new();
    for _ in 0..count {
        chunks.push(iter.take(chunk_len).collect())
    }
    chunks
}

pub fn subgraph_to_sns<S>(sns_client: &S, mut subgraphs: Graph) -> Result<(), Error>
    where S: Sns
{
    let mut proto = Vec::with_capacity(8192);
    let mut compressed = Vec::with_capacity(proto.len());

    for nodes in chunk(subgraphs.nodes, 1000) {
        proto.clear();
        compressed.clear();
        let mut edges = HashMap::new();
        for node in nodes.keys() {
            let node_edges = subgraphs.edges.remove(node);
            if let Some(node_edges) = node_edges {
                edges.insert(node.to_owned(), node_edges);
            }
        }
        let subgraph = Graph {
            nodes,
            edges,
            timestamp: subgraphs.timestamp
        };

        subgraph.encode(&mut proto)?;

        let subgraph_merged_topic_arn = std::env::var("SUBGRAPH_MERGED_TOPIC_ARN").expect("SUBGRAPH_MERGED_TOPIC_ARN");

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
    }

    // If we still have edges, but the nodes were not part of the subgraph, emit those as another event
    if !subgraphs.edges.is_empty() {
        for edges in chunk(subgraphs.edges, 1000) {
            proto.clear();
            compressed.clear();
            let subgraph = Graph {
                nodes: HashMap::new(),
                edges,
                timestamp: subgraphs.timestamp
            };

            subgraph.encode(&mut proto)?;

            let subgraph_merged_topic_arn = std::env::var("SUBGRAPH_MERGED_TOPIC_ARN").expect("SUBGRAPH_MERGED_TOPIC_ARN");

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

        }
    }

    Ok(())
}


#[derive(Clone)]
struct GraphMerger {
    mg_alphas: Vec<String>,
}

async fn upsert_edge(mg_client: &DgraphClient, mu: api::Mutation) -> Result<(), Error> {
    let mut txn = mg_client.new_txn();
    let upsert_res = txn.mutate(mu).await?;

    txn.commit_or_abort().await?;

    Ok(())
}

async fn async_handler(mg_client: DgraphClient, subgraph: Graph) -> Result<(), Error> {
    let mut upsert_res = None;
    let mut edge_res = None;

    let mut node_key_to_uid = HashMap::new();

    let upserts = subgraph.nodes.values().map(|node| {
        upsert_node(&mg_client, node.clone()).map(move |u| (node.get_node_key(), u))
    });

    let upserts = log_time!("All upserts", join_all(upserts).await);

    for (node_key, upsert) in upserts {
        let new_uid = match upsert  {
            Ok(new_uid) => new_uid,
            Err(e) => {  warn!("{}", e); upsert_res = Some(e); continue}
        };

        node_key_to_uid.insert(node_key, new_uid);
    }

    if node_key_to_uid.is_empty() {
        bail!("Failed to attribute uids to any of {} node keys", subgraph.nodes.len());
    }

    info!("Upserted: {} nodes", node_key_to_uid.len());

    info!("Inserting edges {}", subgraph.edges.len());

    let edge_mutations: Vec<_> = subgraph.edges
        .values()
        .map(|e| &e.edges)
        .flatten()
        .filter_map(|edge| {
            match (node_key_to_uid.get(&edge.from[..]), node_key_to_uid.get(&edge.to[..])) {
                (Some(from), Some(to)) if from == to => {
                    let err =
                        format!(
                            "From and To can not be the same uid {} {} {} {} {}",
                            from,
                            to,
                            &edge.from[..],
                            &edge.to[..],
                            &edge.edge_name);
                    error!("{}", err);
                    edge_res = Some(
                        err
                    );
                    None
                }
                (Some(from), Some(to)) => {
                    info!("Upserting edge: {} {} {}", &from, &to, &edge.edge_name);
                    Some(generate_edge_insert(&from, &to, &edge.edge_name))
                }
                (_, _) => {
                    edge_res = Some("Edge to uid failed".to_string()); None
                }
            }
        })
        .map(|mu| upsert_edge(&mg_client, mu))
        .collect();

    let _: Vec<_> = join_all(edge_mutations).await;

    // If our node_key_to_uid map isn't empty we must have merged at least a single node,
    // so even if all edges failed, or even if some upserts failed, we should output the graph
    let region = {
        let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
        Region::from_str(&region_str).expect("Invalid Region")
    };

    let sns_client = SnsClient::new(region);


    if let Err(e) = subgraph_to_sns(&sns_client, subgraph) {
        bail!("subgraph sns publish failed: {}", e)
    };

    if let Some(err) = upsert_res {
        error!("{}", err);
        return Err(err)
    };
    if let Some(err) = edge_res {
        error!("{}", err);
        bail!(err);
    };

    Ok(())

}

impl EventHandler<Graph> for GraphMerger {
    fn handle_event(&self, subgraph: Graph) -> Result<(), Error> {
        if subgraph.is_empty() {
            warn!("Attempted to merge empty subgraph. Short circuiting.");
            return Ok(())
        }

        println!("handling new subgraph with {} nodes {} edges", subgraph.nodes.len(), subgraph.edges.len());

        let mut rng = thread_rng();
        let rand_alpha = self.mg_alphas.choose(&mut rng)
            .expect("Empty rand_alpha");

        let mg_client = DgraphClient::new(
            vec![
                api_grpc::DgraphClient::with_client(
                    Arc::new(
                        Client::new_plain(rand_alpha, 9080, ClientConf {
                            ..Default::default()
                        })?
                    )
                )
            ]
        );

        async_std::task::block_on(async_handler(mg_client, subgraph))
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
