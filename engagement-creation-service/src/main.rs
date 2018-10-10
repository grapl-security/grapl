#![feature(nll)]
extern crate aws_lambda as lambda;
extern crate dgraph_client;
extern crate engagements;
extern crate incident_graph;
extern crate failure;
#[macro_use]
extern crate log;
extern crate graph_descriptions;

extern crate prost;
extern crate sqs_microservice;

#[macro_use] extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate uuid;

use failure::Error;
use dgraph_client::api_grpc::DgraphClient;
use engagements::*;
use incident_graph::*;
use lambda::event::sqs::SqsEvent;
use prost::Message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::channel;
use uuid::Uuid;
use sqs_microservice::handle_sns_sqs_json;

use incident_graph::*;


use graph_descriptions::graph_description::*;
use graph_descriptions::*;


fn expand_and_contextualize(
    root_nodes: &mut Vec<RootNode>,
    dgraph_client: &DgraphClient
) {

    let mut expanded = HashSet::new();

    for node in root_nodes.iter_mut() {
        let node = match node {
            RootNode::Process(process) => expand_process(&dgraph_client, process, &mut expanded),
            RootNode::File(file) => unimplemented!() ,//expand_file(&dgraph_client, file, &mut expanded),
        };
    }

}


fn expand_file(dgraph_client: &DgraphClient,
               file: &mut File,
               expanded: &mut HashSet<String>)
{
    info!("Expanding file");
    if expanded.len() >= 100 {
        return
    }

    if expanded.get(&file.uid).is_some() {
        return
    }

    file.add_creator(dgraph_client);

    File::procs_executed_from(&file, dgraph_client);
    expanded.insert(file.uid.clone());

}

// Given a process node, and it's state, get the related nodes
// and add them to the engagement graph
fn expand_process(dgraph_client: &DgraphClient,
                  process: &mut Process,
                  expanded: &mut HashSet<String>
                 )
{
    info!("Expanding process");
    if expanded.len() >= 100 {
        return;
    }

    if expanded.get(&process.uid).is_some() {
        info!("Reached a process we've already expanded.");
        return
    }

    expanded.insert(process.uid.clone());

    if process.bin_file.is_none() {
        info!("Expanding process bin_file");
        process.add_file(dgraph_client);
        if let Some(bin_file) = process.bin_file.as_mut() {
            expand_file(dgraph_client, bin_file, expanded);
        }
    }

    if process.children.is_empty() {
        info!("Adding child to process");
        process.add_children(dgraph_client);
    }
    info!("Expanding process children");
    for child_proc in process.children.iter_mut() {
        expand_process(dgraph_client, child_proc, expanded);
    }
}


fn handle_incident(mut root_nodes: Vec<RootNode>, engagement_key: String) {
    info!("Connecting to master");
    let master_client = dgraph_client::new_client("db.mastergraph:9080");
    info!("Connecting to engagement");
    let engagement_client = dgraph_client::new_client("db.engagementgraph:9080");

    // Uuid to represent our engagement

    // Get all of the context from the IG
    info!("Expanding routes");
    expand_and_contextualize(
        &mut root_nodes,
        &master_client
    );

    info!("Inserting {} nodes", root_nodes.len());
    for node in root_nodes {

        let mut node_value  = match node {
            RootNode::Process(process) => {
                serde_json::to_value(&process)
                    .expect("to_value(&process)")
            }
            RootNode::File(file) => {
                serde_json::to_value(&file)
                    .expect("to_value(&file)")
            }
        };

        node_value["engagement_key"] = engagement_key.clone().into();
        let encoded_node = serde_json::to_vec(&node_value).expect("node_value to_vec");

        let mut mutation = dgraph_client::api::Mutation::new();

        mutation.set_json = encoded_node;
        mutation.commit_now = true;
        let mut_res = engagement_client.mutate(&mutation);

        info!("inserted nodes {:#?}", mut_res);
//        insert_root_node(
//            &engagement_client,
//            &engagement_key,
//            node
//        );
    }

}


#[derive(Serialize, Deserialize)]
struct Incident {
    root_nodes: RootNode,
}

fn main() {

    handle_sns_sqs_json(move |incidents: Vec<RootNode>| {
        let mut dgraph_client =
            dgraph_client::new_client("db.engagementgraph:9080");

        set_engagement_process_schema(&dgraph_client);
        set_engagement_file_schema(&dgraph_client);


        info!("Handling incident");


        let mut new_incidents = vec![];
        for incident in incidents.into_iter() {
            if !subgraph_exists(&dgraph_client, &incident) {
                info!("not throttling");
                new_incidents.push(incident);
            } else {
                info!("throttling");
            }
        }

        if !new_incidents.is_empty() {
            let engagement_key = Uuid::new_v4().to_string();
            handle_incident(new_incidents, engagement_key);
        }

        Ok(())
    },
    move |()| {
        Ok(())
    })
}
