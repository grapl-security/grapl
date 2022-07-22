#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();

    let config = grapl_web_ui::ConfigBuilder::from_env()?.build()?;

    grapl_web_ui::run(config)?.await?;

    // send remaining trace spans.
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
