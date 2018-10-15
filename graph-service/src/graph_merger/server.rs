#![feature(test, nll, proc_macro_non_items, generators, async_await, use_extern_macros)]

extern crate aws_lambda as lambda;
extern crate base64;
extern crate bytes;
extern crate dgraph_client;
extern crate env_logger;
extern crate futures;
extern crate graph_descriptions;
extern crate graph_merger;
#[macro_use]
extern crate log;
extern crate prost;
#[macro_use]
extern crate prost_derive;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate rusoto_core;
extern crate rusoto_sns;
extern crate rusoto_sqs;
extern crate serde;
extern crate sqs_microservice;
extern crate stopwatch;
extern crate tokio_core;

use futures::Future;
use graph_descriptions::graph_description::GraphDescriptionProto;
use graph_merger::{merge_subgraph, set_file_schema, set_process_schema};
use prost::Message;
use rusoto_core::region::Region;
use rusoto_sns::{Sns, SnsClient};
use rusoto_sns::PublishInput;
use rusoto_sns::CreateTopicInput;
use sqs_microservice::handle_s3_sns_sqs_proto;
use stopwatch::Stopwatch;
use subgraph_merge_event::SubgraphMerged;

macro_rules! log_time {
    ($msg: expr, $x:expr) => {
        let mut sw = Stopwatch::start_new();
        let result = {$x};
        sw.stop();
        info!("{} {} milliseconds", $msg, sw.elapsed_ms());
        result
    };
}

mod subgraph_merge_event {
    include!(concat!(env!("OUT_DIR"), "/subgraph_merge_event.rs"));
}



/// The Graph Merge Service
#[cfg_attr(tarpaulin, skip)]
pub fn main() {

    handle_s3_sns_sqs_proto(move |subgraph: GraphDescriptionProto| {
        println!("connecting to db.mastergraph");
        let mut dgraph_client =
            dgraph_client::new_client("db.mastergraph:9080");
        println!("connected to db.mastergraph");
        set_process_schema(&mut dgraph_client);
        set_file_schema(&mut dgraph_client);

        info!("Set schemas");
        log_time!{
            "merge_subgraph",
            merge_subgraph(&dgraph_client, &subgraph.into())
        }

    }, move |(earliest, latest)| {
        let event = SubgraphMerged {
            earliest,
            latest,
        };

        let mut buf = Vec::new();
        event.encode(&mut buf)?;
        let event = buf;

        let event = base64::encode(&event);

        log_time!{
            "merge_subgraph",
            {
            let sns_client = SnsClient::simple(
                Region::UsEast1
            );

            let arn = sns_client.create_topic(
                &CreateTopicInput {
                    name: "subgraphs-merged-topic".into()
                }
            ).wait()?.topic_arn
                .expect("arn was none for subgraphs-merged-topic");

            info!("Got arn for subgraphs-merged-topic");

            info!("Publishing {} bytes to SNS", event.len());
            sns_client.publish(
                &PublishInput {
                    message: event,
                    topic_arn: arn.to_string().into(),
                    ..Default::default()
                }
            ).wait()?;
            }

        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use graph_descriptions::*;
    use stopwatch::Stopwatch;

    use super::*;


    #[test]
    fn test_empty_merge() {
        let subgraph = GraphDescription::new(1);

        let mut client =
            dgraph_client::new_client("localhost:9080");

        let f = merge_subgraph(
            &client,
            &subgraph.into()
        );

        f.unwrap();
    }

    quickcheck! {
        fn test_single_process_create_merge(
                asset_id: String,
                pid: u64,
                timestamp: u64,
                image_name: Vec<u8>,
                image_path: Vec<u8>
            ) -> bool
        {
            let asset_id = graph_descriptions::HostIdentifier::AssetId(asset_id);
            let pid = pid + 1;
            let process_state = ProcessState::Created;

            let mut client =
                dgraph_client::new_client("localhost:9080");

            set_process_schema(&mut client);

            let process_description = ProcessDescription::new (
                asset_id.clone(),
                process_state.clone(),
                pid.clone(),
                timestamp.clone(),
                image_name.clone(),
                image_path.clone(),
            );

            let mut subgraph = GraphDescription::new(1);
            subgraph.add_node(process_description);

            let mut client =
                dgraph_client::new_client("localhost:9080");

            let f = merge_subgraph(
                &client,
                &subgraph.into()
            );


            f.unwrap();

            let mut req = dgraph_client::api::Request::new();

            let mut client =
                dgraph_client::new_client("localhost:9080");

            req.query = format!(r#"
                {{
                    question(func: eq(pid, "{}"))
                    @filter(eq(create_time, {}))
                    {{
                        uid,
                        pid,
                        create_time,
                        node_key
                    }}
                }}"#, pid, timestamp).to_string();

            let resp = client.query(&req).expect("query");

            let resp: serde_json::Value = serde_json::from_slice(resp.get_json()).unwrap();

            let uid = resp["question"][0].get("uid").map(|uid| uid.as_str().unwrap()).clone();
            let r_pid = resp["question"][0].get("pid").map(|uid| uid.as_u64().unwrap()).clone();
            let r_create_time = resp["question"][0].get("create_time").map(|uid| uid.as_u64().unwrap()).clone();
            let node_key = resp["question"][0].get("node_key").map(|uid| uid.as_str().unwrap()).clone();

            let uid  = uid.expect("uid");
            let r_pid  = r_pid.expect("pid");
            let r_create_time  = r_create_time.expect("create_time");
            let node_key  = node_key.expect("node_key");

            pid == r_pid && timestamp == r_create_time

        }

    }


    // This test is going to test a successful merge of a parent and child process
    // We'll perform the merge_subgraph function, then validate that we can query
    // by the parent's node_key and get the child process that we expect
    quickcheck! {
        fn test_process_child_create_merge(
                asset_id: String,
                pid: u64,
                timestamp: u64,
                image_name: Vec<u8>,
                image_path: Vec<u8>
            ) -> bool
        {
            let asset_id = graph_descriptions::HostIdentifier::AssetId(asset_id);

            let child_pid = pid + 20;
            let child_create_time = timestamp + 200;
            let process_state = ProcessState::Created;

            let mut client =
                dgraph_client::new_client("localhost:9080");

            set_process_schema(&mut client);

            let parent_process = ProcessDescription::new (
                asset_id.clone(),
                process_state.clone(),
                pid.clone(),
                timestamp.clone(),
                image_name.clone(),
                image_path.clone(),
            );

            let parent_process_node_key = parent_process.clone_key();

            let child_process = ProcessDescription::new (
                asset_id.clone(),
                process_state.clone(),
                child_pid.clone(),
                child_create_time.clone(),
                image_name.clone(),
                image_path.clone(),
            );
            let child_process_node_key = child_process.clone_key();

            let mut subgraph = GraphDescription::new(1);
            subgraph.add_edge("children",
                              parent_process.clone_key(),
                              child_process.clone_key());
            subgraph.add_node(parent_process);
            subgraph.add_node(child_process);

            let mut client =
                dgraph_client::new_client("localhost:9080");

            let mut sw = stopwatch::Stopwatch::start_new();

            let f = merge_subgraph(
                &client,
                &subgraph.into()
            );

            println!("merge_subgraph took {}ms", sw.elapsed_ms());

            f.unwrap();

            let mut req = dgraph_client::api::Request::new();

            let mut client =
                dgraph_client::new_client("localhost:9080");

            req.query = format!(r#"
                {{
                    question(func: eq(node_key, {}))
                    {{
                        uid,
                        pid,
                        create_time,
                        node_key,
                        children
                        @filter(eq(node_key, {}))
                        {{
                            uid,
                            pid,
                            create_time,
                            node_key,
                        }}
                    }}
                }}"#, parent_process_node_key, child_process_node_key).to_string();

            let resp = client.query(&req).expect("query");

            let resp: serde_json::Value = serde_json::from_slice(resp.get_json()).unwrap();

            assert_eq!(resp["question"].as_array().unwrap().len(), 1,
                "incorrect number of responses to question");

            let uid = resp["question"][0].get("uid").map(|uid| uid.as_str().unwrap()).clone();
            let r_pid = resp["question"][0].get("pid").map(|uid| uid.as_u64().unwrap()).clone();
            let r_create_time = resp["question"][0].get("create_time").map(|uid| uid.as_u64().unwrap()).clone();
            let node_key = resp["question"][0].get("node_key").map(|uid| uid.as_str().unwrap()).clone();

            let uid  = uid.expect("uid");
            let r_pid  = r_pid.expect("pid");
            let r_create_time  = r_create_time.expect("create_time");
            let node_key  = node_key.expect("node_key");

            let child = resp["question"][0].get("children").map(|uid| uid.as_array().unwrap()).clone();
            let child  = child.expect("child");

            assert_eq!(child.len(), 1);

            pid == r_pid && timestamp == r_create_time
        }

    }

}


