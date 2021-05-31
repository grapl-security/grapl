#![allow(warnings)]
use::reqwest;
use actix_web::{HttpResponse, Result};

use reqwest::Body;
use serde::{Serialize, Deserialize};
use crate::model_plugin_deployer_router::routes::{DeployRequest, CustomError};
use crate::auth_router::routes::{LoginBody, AuthError};
use serde::__private::Result::Ok;
use serde::__private::Vec;

pub mod model_plugin_deployer_router;
pub mod auth_router;

#[derive(Serialize, Deserialize)]
pub struct PluginObject {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct PluginList {
    plugin_list: Vec<PluginObject>,
}


#[derive(Serialize, Deserialize)]
pub struct AuthBody{
    username: String,
    password: String,
}

pub async fn login_request_with_body(path: &str, body: LoginBody) ->  Result<AuthBody, AuthError> { // dyn, dynamic, we don't know what type
    let client = reqwest::Client::new();

    let response: AuthBody = client.post(format!("http://localhost:8000/login/{}", path)) // we need to change this
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    return Ok(response)
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


#[cfg(test)]
mod tests {
    #[test]
    fn get_body() {
        assert_eq!(2 + 2)
    }
}