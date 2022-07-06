use std::error::Error;

// hook to grpc
// write off to kafka
// pipeline ingress uses kafka crate src/rust

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Testing");
    Ok(())
}