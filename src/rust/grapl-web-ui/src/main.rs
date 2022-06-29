mod authn;
mod config;
mod routes;
mod services;

use actix_session::CookieSession;
use actix_web::{
    web,
    web::Data,
    App,
    HttpServer,
};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();

    let config = config::Config::from_env()?;

    let bind_address = config.bind_address.clone();

    HttpServer::new(move || {
        let web_client = Data::new(awc::Client::new());
        let auth_dynamodb_client = Data::new(authn::AuthDynamoClient {
            client: config.dynamodb_client.clone(),
            user_auth_table_name: config.user_auth_table_name.clone(),
            user_session_table_name: config.user_session_table_name.clone(),
        });
        let graphql_endpoint = Data::new(config.graphql_endpoint.clone());
        let model_plugin_deployer_endpoint =
            Data::new(config.model_plugin_deployer_endpoint.clone());

        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web_opentelemetry::RequestTracing::new())
            .wrap(actix_web::middleware::Compress::default())
            .wrap(
                CookieSession::private(&config.session_key)
                    .path("/")
                    .secure(true),
            )
            // Content-Security-Policy
            .wrap(actix_web::middleware::DefaultHeaders::new()
                .add((actix_web::http::header::CONTENT_SECURITY_POLICY, "Content-Security-Policy-Report-Only: script-src https://accounts.google.com/gsi/client; frame-src https://accounts.google.com/gsi/; connect-src https://accounts.google.com/gsi/;"))
            )
            .app_data(web_client)
            .app_data(graphql_endpoint)
            .app_data(model_plugin_deployer_endpoint)
            .app_data(auth_dynamodb_client)
            .configure(routes::config)
            .service(web::scope("/graphQlEndpoint").configure(services::graphql::config))
            .service(
                web::scope("/modelPluginDeployer")
                    .configure(services::model_plugin_deployer::config),
            )
    })
    .bind(&bind_address)?
    .run()
    .await?;

    // send remaining trace spans.
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
