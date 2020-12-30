#![type_length_limit = "1195029"]

use log::{error, info};
use std::time::Duration;

use node_identifier::handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = grapl_config::init_grapl_env!();
    info!("Running node-identifier");

    handler(false).await?;

    Ok(())
}
