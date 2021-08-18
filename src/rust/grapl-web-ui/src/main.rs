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
use tap::TapFallible;

#[derive(thiserror::Error, Debug)]
enum GraplUiError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("IO error")]
    Io(#[from] std::io::Error),
}

#[actix_web::main]
async fn main() -> Result<(), GraplUiError> {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env().tap_err(|e| tracing::error!("{}", e))?;

    let bind_address = config.bind_address.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Compress::default())
            .wrap(
                // CookieSession::private(&config.session_key)
                CookieSession::signed(&config.session_key)
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

    Ok(())
}
