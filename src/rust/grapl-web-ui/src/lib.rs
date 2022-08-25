mod authn;
mod config;
// mod graphql;
pub mod routes;
mod upstream;

use std::sync::Mutex;

use actix_session::CookieSession;
use actix_web::{
    dev::Server,
    web::Data,
    App,
    HttpServer,
};
pub use config::Config;

pub fn run(config: config::Config) -> Result<Server, std::io::Error> {
    let listener = config.listener;

    let server = HttpServer::new(move || {
        let web_client = Data::new(awc::Client::new());
        let web_authenticator = Data::new(authn::WebAuthenticator::new(
            authn::AuthDynamoClient::new(
                config.dynamodb_client.clone(),
                config.user_auth_table_name.clone(),
                config.user_session_table_name.clone(),
            ),
            jsonwebtoken_google::Parser::new(&config.google_client_id),
        ));
        let graphql_endpoint = Data::new(config.graphql_endpoint.clone());
        let plugin_registry_client = Data::new(Mutex::new(config.plugin_registry_client.clone()));

        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web_opentelemetry::RequestTracing::new())
            .wrap(actix_web::middleware::Compress::default())
            .wrap(
                CookieSession::private(&config.session_key)
                    .path("/")
                    .secure(true),
            )
            .wrap(actix_web::middleware::DefaultHeaders::new().add((
                actix_web::http::header::CONTENT_SECURITY_POLICY_REPORT_ONLY,
                concat!(
                    "img-src 'self'; ",
                    "script-src-elem 'self' https://accounts.google.com/gsi/client; ",
                    "frame-src https://accounts.google.com/gsi/; ",
                    "connect-src 'self' https://accounts.google.com/gsi/; ",
                    "frame-ancestors 'self'; ",
                    "form-action 'self'; "
                ),
            )))
            .app_data(web_client)
            .app_data(plugin_registry_client)
            .app_data(graphql_endpoint)
            .app_data(web_authenticator)
            .configure(routes::config)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
