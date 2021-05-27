use actix_web::HttpServer;
use grapl_router_api::model_plugin_deployer_router::routes::grapl_model_plugin_deployer;

type StdErr = Box<dyn std::error::Error>;

#[actix_web::main]// tell actix-web to start a runtime and execute main functions as first task
async fn main() -> Result<(), StdErr> {
    // set up tracing
    // set up integration tests

        HttpServer::new(move || {
                actix_web::App::new()
                    .service(grapl_model_plugin_deployer)
            })
            .bind(("127.0.0.1", 8000))?
            .run()
            .await?;

            Ok(())
}


