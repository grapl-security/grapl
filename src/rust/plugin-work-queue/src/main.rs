use plugin_work_queue::server::exec_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    exec_service().await?;
    Ok(())
}
