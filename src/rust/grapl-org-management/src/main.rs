use grapl_org_management::server;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start_server().await?;
    println!("Test");
    Ok(())
}
