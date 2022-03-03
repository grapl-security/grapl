mod authn;
mod config;
mod routes;
mod services;

use actix_session::CookieSession;
use actix_web::{
    client::Client,
    web,
    App,
    HttpServer,
};
use actix_web_opentelemetry::RequestTracing;
use opentelemetry::{global, trace::TraceError};
use opentelemetry::sdk::trace::Config;
use opentelemetry::sdk::Resource;
use tap::TapFallible;

#[derive(thiserror::Error, Debug)]
enum GraplUiError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Trace Error")]
    Trace(#[from] TraceError),
}

#[actix_web::main]
async fn main() -> Result<(), GraplUiError> {
    let (_env, _guard) = grapl_config::init_grapl_env!();

    let config = config::Config::from_env().tap_err(|e| tracing::error!("{}", e))?;

    let bind_address = config.bind_address.clone();

    // Start an otel jaegar trace pipeline
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    opentelemetry_jaeger::new_pipeline()
        .with_service_name("grapl-web-ui")
        .with_trace_config(Config::default().with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "web-ui"),
            opentelemetry::KeyValue::new("exporter", "otlp-jaeger"),
        ])))
        .with_collector_endpoint("http://100.115.92.202:14268/api/traces")
        .install_simple()?;
        // TODO switch to batch once we upgrade to actix-web 4, which supports Tokio 1.x
        //.install_batch(opentelemetry::runtime::Tokio)?;

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(RequestTracing::new())
            // grapl-security/issue-tracker#803
            // .wrap(actix_web::middleware::Compress::default())  // todo: Reenable compression when brotli isn't vulnerable
            .wrap(
                CookieSession::private(&config.session_key)
                    .path("/")
                    .secure(true),
            )
            .data(Client::new())
            .data(config.graphql_endpoint.clone())
            .data(config.model_plugin_deployer_endpoint.clone())
            .data(authn::AuthDynamoClient {
                client: config.dynamodb_client.clone(),
                user_auth_table_name: config.user_auth_table_name.clone(),
                user_session_table_name: config.user_session_table_name.clone(),
            })
            .configure(routes::config)
            .service(web::scope("/graphQlEndpoint").configure(services::graphql::config))
            .service(
                web::scope("/modelPluginDeployer")
                    .configure(services::model_plugin_deployer::config),
            )
    })
    .bind(bind_address)?
    .run()
    .await?;

    // sending remaining spans. Do we need this?
    global::shutdown_tracer_provider();

    Ok(())
}
