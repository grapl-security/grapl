#![allow(warnings)]
use std::error::Error;
use tonic::transport::Server;
use tracing::info;



// hook to grpc
// write off to kafka
// pipeline ingress uses kafka crate src/rust

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {


    Ok(())
}