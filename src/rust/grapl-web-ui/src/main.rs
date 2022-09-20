use grapl_tracing::setup_tracing;

const SERVICE_NAME: &'static str = "grapl-web-ui";

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;

    let config = grapl_web_ui::Config::from_env().await?;

    grapl_web_ui::run(config)?.await?;

    Ok(())
}
