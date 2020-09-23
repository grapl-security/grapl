use serde::{Deserialize, Serialize};
use graph_descriptions::graph_description::*;
use graph_descriptions::process::ProcessState;
use graph_descriptions::node::NodeT;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessStop {
    process_id: u64,
    name: String,
    hostname: String,
    timestamp: u64,
    eventname: String,
}

impl From<ProcessStop> for Graph {
    fn from(process_stop: ProcessStop) -> Self {
        let terminated_process = ProcessBuilder::default()
            .process_name(process_stop.name)
            .hostname(process_stop.hostname)
            .state(ProcessState::Terminated)
            .process_id(process_stop.process_id)
            .terminated_timestamp(process_stop.timestamp)
            .build()
            .unwrap();

        let mut graph = Graph::new(process_stop.timestamp);
        graph.add_node(terminated_process);

        graph
    }
}