use {{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}::get_socket_addr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _subscriber = ::tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    let socket_addr = get_socket_addr().parse()?;

    {{cookiecutter.snake_project_name}}::server::exec_service(socket_addr).await?;
    Ok(())
}
