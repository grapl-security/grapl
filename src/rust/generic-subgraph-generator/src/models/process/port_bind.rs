use serde::{Deserialize, Serialize};
use graph_descriptions::graph_description::*;
use graph_descriptions::process::ProcessState;
use graph_descriptions::file::FileState;
use graph_descriptions::node::NodeT;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessPortBindLog {
    pid: u64,
    bound_port: u64,
    hostname: String,
    timestamp: u64,
    eventname: String,
}