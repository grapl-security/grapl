use plugin_registry::{
    plugin_registry::get_socket_addr,
    server::exec_service,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let socket_addr = get_socket_addr()?;
    exec_service(socket_addr).await?;
    Ok(())
}
