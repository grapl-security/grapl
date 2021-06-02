use actix_web::HttpServer;
use grapl_router_api::model_plugin_deployer_router::routes::grapl_model_plugin_deployer;
use grapl_router_api::auth_router::routes::grapl_login;
use grapl_router_api::graphql_router::routes::graphql_router;
use tracing_actix_web::TracingLogger;

type StdErr = Box<dyn std::error::Error>;

#[actix_web::main]
async fn main() -> Result<(), StdErr> {
    // set up tracing
    // set up integration tests
        HttpServer::new(move || {
                actix_web::App::new()
                    .wrap(TracingLogger::default())
                    .service(grapl_model_plugin_deployer)
                    .service(grapl_login)
                    .service(graphql_router)
            })
            .bind(("127.0.0.1", 8000))?
            .run()
            .await?;

            Ok(())
}


