use org_management::server;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let (_env, _guard) = grapl_config::init_grapl_env!();
    println!("TESTING: TODO REMOVE THIS println");
    tracing::info!(message="Started org-manager");

    server::start_server()
        .await?;
    
    Ok(())
}
