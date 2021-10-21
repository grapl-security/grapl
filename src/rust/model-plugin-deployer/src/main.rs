use model_plugin_deployer::model_plugin_deployer::get_socket_addr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();

    let socket_addr = get_socket_addr()?;

    model_plugin_deployer::server::exec_service(socket_addr).await?;
    Ok(())
}
