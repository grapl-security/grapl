extern crate lambda_runtime as lambda;
extern crate node_identifier;
extern crate simple_logger;

use node_identifier::{retry_handler, local_handler};

use lambda::lambda;

// fn main() {
//     simple_logger::init_with_level(log::Level::Info).unwrap();
//     lambda!(retry_handler);
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    // lambda!(handler);
    local_handler(true).await?;
    Ok(())
}
