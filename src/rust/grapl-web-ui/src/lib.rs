use ::reqwest;
use actix_web::Result;
use serde::{
    de::DeserializeOwned,
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
        DeployRequest,
        PluginError,
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

pub async fn do_request<B, R, I, E>(path: &str, body: B) -> Result<R, E>
where
    B: Into<Option<I>>,
    R: DeserializeOwned,
    I: Serialize,
    E: From<reqwest::Error>,
{
    let client = reqwest::Client::new();

    let mut request_builder = client.post(format!("http://localhost:8000/{}", path));

    if let Some(body) = body.into() {
        request_builder = request_builder.json(&body);
    }

    let response = request_builder.send().await?;

    let deserialized_response = response.json().await?;

    Ok(deserialized_response)
}

pub async fn graphql_request(path: &str, body: GraphQLBody) -> Result<GraphQLBody, GraphQLError> {
    do_request(path, body).await
}

pub async fn login_request_with_body(path: &str, body: LoginBody) -> Result<AuthBody, AuthError> {
    do_request(path, body).await
}

pub async fn request_with_body(
    path: &str,
    body: DeployRequest,
) -> Result<PluginObject, PluginError> {
    do_request(path, body).await
}

pub async fn make_request(path: &str) -> Result<PluginList, PluginError> {
    do_request::<_, _, (), _>(path, None).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_body() {
        let test_var = 2 + 2;
        assert_eq!(test_var, 4);
    }
}
