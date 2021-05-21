// use grapl_router_api::model_plugin_deployer_router::delete::grapl_model_plugin_delete;

//
// pub fn get_path(_path: String) -> String {
//     // if path == "/modelPluginDeployer"{
//     //     deploy();
//     // }
//     //
//     // if path == "/deleteModelPlugin"{
//     //     delete();
//     // }
//     //
//     // if path == "list" {
//     //     list();
//     // }
//     return "test".to_string();
//
// }

// use actix_web::HttpRequest;
// use std::thread::Builder;
// use std::thread::Builder;
// use http::{Request, Response}; // this library just describes types
// use ^^ to implement the http client but it's not a client itself

use::reqwest;

pub async fn make_request(path: &str) ->  Result<(), Box<dyn std::error::Error>>{
    let client = reqwest::Client::new();
    let response = client.post(format!("http://localhost:8000/{}", path))
        .body("tbd")
        .send()
        .await?;

    Ok(())
}

pub mod model_plugin_deployer_router;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}