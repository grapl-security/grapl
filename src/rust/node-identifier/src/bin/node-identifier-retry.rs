#![type_length_limit = "1195029"]

use node_identifier::handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    handler(true).await
}
