extern crate lambda_runtime as lambda;
extern crate node_identifier;
extern crate simple_logger;

use log::{info, error};

use node_identifier::{handler, local_handler};

use lambda::lambda;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let is_local = std::env::var("IS_LOCAL")
        .is_ok();

    if is_local {
        info!("Running locally");
        std::thread::sleep_ms(10_000);
        let mut runtime = Runtime::new().unwrap();

        loop {
            if let Err(e) = runtime.block_on(async move { local_handler(true).await }) {
                error!("{}", e);
            }

            std::thread::sleep_ms(2_000);
        }
    }  else {
        info!("Running in AWS");
        lambda!(handler);
    }
    Ok(())
}
