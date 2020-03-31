extern crate lambda_runtime as lambda;
extern crate node_identifier;
extern crate simple_logger;

use node_identifier::{handler, local_handler};

use lambda::lambda;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    // lambda!(handler);
    loop {
        local_handler(false).await;
    }
    Ok(())
}
