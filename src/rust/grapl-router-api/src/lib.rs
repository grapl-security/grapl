use ::reqwest;
use actix_web::Result;
use serde::{
    Deserialize,
    Serialize,
    __private::{
        Result::Ok,
        Vec,
    },
};

use crate::{
    auth_router::routes::{
        AuthError,
        LoginBody,
    },
    graphql_router::routes::{
        GraphQLBody,
        GraphQLError,
    },
    model_plugin_deployer_router::routes::{
        CustomError,
        DeployRequest,
    },
};
pub mod auth_router;
pub mod graphql_router;
pub mod model_plugin_deployer_router;

#[derive(Serialize, Deserialize)]
pub struct PluginObject {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct PluginList {
    plugin_list: Vec<PluginObject>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthBody {
    username: String,
    password: String,
}

pub async fn graphql_request(path: &str, body: GraphQLBody) -> Result<GraphQLBody, GraphQLError> {
    // dyn, dynamic, we don't know what type
    let client = reqwest::Client::new();

    let response: GraphQLBody = client
        .post(format!("http://localhost:8000/graphql/{}", path)) // we need to change this
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    return Ok(response);
}

pub async fn login_request_with_body(path: &str, body: LoginBody) -> Result<AuthBody, AuthError> {
    // dyn, dynamic, we don't know what type
    let client = reqwest::Client::new();

    let response: AuthBody = client
        .post(format!("http://localhost:8000/login/{}", path)) // we need to change this
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    return Ok(response);
}

pub async fn request_with_body(
    path: &str,
    body: DeployRequest,
) -> Result<PluginObject, CustomError> {
    // dyn, dynamic, we don't know what type
    let client = reqwest::Client::new();

    let response: PluginObject = client
        .post(format!(
            "http://localhost:8000/modelPluginDeployer/{}",
            path
        )) // we need to change this
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    return Ok(response);
}

pub async fn make_request(path: &str) -> Result<PluginList, CustomError> {
    let client = reqwest::Client::new();

    let list_response: PluginList = client
        .post(format!(
            "http://localhost:8000/modelPluginDeployer/{}",
            path
        )) // we need to change this
        .send()
        .await?
        .json()
        .await?;

    return Ok(list_response);
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_body() {
        let test_var = 2 + 2;
        assert_eq!(test_var, 4);
    }
}
