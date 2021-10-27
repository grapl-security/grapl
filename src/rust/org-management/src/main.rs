use orgmanagement::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();

    tracing::info!(message = "Started org-manager");
    server::start_server().await?;

    Ok(())
}
