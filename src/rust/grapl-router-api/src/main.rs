use actix_web::{ App, HttpServer};
use grapl_api_service::model_plugin_deployer_router::deploy::grapl_model_plugin_deployer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // set up tracing
    // set up integration tests
    HttpServer::new(|| App::new().service(grapl_model_plugin_deployer))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}