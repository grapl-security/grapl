#![type_length_limit = "1195029"]

use log::{error, info};
use std::time::Duration;

use node_identifier::{init_dynamodb_client, handler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();

    handler(true).await;

    Ok(())
}
