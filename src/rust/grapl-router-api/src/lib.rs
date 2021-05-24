#![allow(warnings)]
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


use::reqwest;
use actix_web::{HttpResponse};

use reqwest::Body;
use serde::{Serialize, Deserialize};
use crate::model_plugin_deployer_router::deploy::DeployRequest;


#[derive(Serialize, Deserialize)]
pub struct PluginObject {
    name: String,
}

pub async fn make_request(path: &str, body: DeployRequest) ->  Result<PluginObject, Box<dyn std::error::Error>> { // dyn, dynamic, we don't know what type
    let client = reqwest::Client::new();


    let response: PluginObject = client.post(format!("http://localhost:8000/modelPluginDeployer/{}", path)) // we need to change this
        .json(&body)
        .send()
        .await?
        .json()
        .await?;


    return Ok(response)
}

pub mod model_plugin_deployer_router;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}