use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessPortBindLog {
    pid: u64,
    bound_port: u64,
    hostname: String,
    timestamp: u64,
}
