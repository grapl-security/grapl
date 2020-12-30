#![type_length_limit = "1195029"]

use std::time::Duration;

use log::{error, info};

use node_identifier::{handler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();
    info!("Running node-identifier-retry-handler");

    handler(true).await?;

    Ok(())
}
