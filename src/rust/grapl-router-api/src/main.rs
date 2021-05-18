//use actix_web::{ App, HttpServer};
use actix_web::dev::HttpServiceFactory;
use grapl_router_api::model_plugin_deployer_router::deploy::grapl_model_plugin_deployer;
mod routes;


#[actix_web::main]// tell actix-web to start a runtime and execute main functions as first task
async fn main() -> std::io::Result<()> {
    // set up tracing
    // set up integration tests
   // HttpServer::new(|| App::new()
    //.service(grapl_model_plugin_deployer))
     //  .service(routes::api())

       // .bind("127.0.0.1:8080")?
        //.run()
        //.await

        actix_web::HttpServer::new(move || {
                actix_web::App::new()
                    .service(routes::api())
            })
            .bind(("127.0.0.1", 8000))?
            .run()
            .await?;

            Ok(())
}