extern crate lambda_runtime as lambda;
extern crate node_identifier;
extern crate simple_logger;

use log::error;

use node_identifier::{retry_handler, local_handler};

use lambda::lambda;
use tokio::runtime::Runtime;

// fn main() {
//     simple_logger::init_with_level(log::Level::Info).unwrap();
//     lambda!(retry_handler);
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let is_local = std::env::var("IS_LOCAL")
        .map(|is_local| is_local == "True")
        .unwrap_or(false);

    if is_local {
        let mut runtime = Runtime::new().unwrap();

        loop {
            if let Err(e) = runtime.block_on(async move { local_handler(true).await }) {
                error!("{}", e);
            }
        }
    }  else {
        lambda!(retry_handler);
    }
    Ok(())
}
