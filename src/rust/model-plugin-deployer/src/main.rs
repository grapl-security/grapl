use model_plugin_deployer::model_plugin_deployer::get_socket_addr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _subscriber = ::tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    let socket_addr = get_socket_addr().parse()?;

    model_plugin_deployer::server::exec_service(socket_addr).await?;
    Ok(())
}
