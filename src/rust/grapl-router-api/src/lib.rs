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
use crate::model_plugin_deployer_router::routes::{DeployRequest, CustomError};


#[derive(Serialize, Deserialize)]
pub struct PluginObject {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct PluginList {
    plugin_list: Vec<PluginObject>,
}


pub async fn request_with_body(path: &str, body: DeployRequest) ->  Result<PluginObject, CustomError> { // dyn, dynamic, we don't know what type
    let client = reqwest::Client::new();


    let response: PluginObject = client.post(format!("http://localhost:8000/modelPluginDeployer/{}", path)) // we need to change this
        .json(&body)
        .send()
        .await?
        .json()
        .await?;


    return Ok(response)
}

pub async fn make_request(path: &str) ->  Result<PluginList, CustomError> {
    let client = reqwest::Client::new();

    let list_response: PluginList = client.post(format!("http://localhost:8000/modelPluginDeployer/{}", path)) // we need to change this
        .send()
        .await?
        .json()
        .await?;

    return Ok(list_response)
}


pub mod model_plugin_deployer_router;

#[cfg(test)]
mod tests {
    #[test]
    fn get_body() {
        assert_eq!(2 + 2)
    }
}