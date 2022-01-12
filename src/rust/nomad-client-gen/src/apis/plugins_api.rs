/*
 * Nomad
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.1.4
 * Contact: support@hashicorp.com
 * Generated by: https://openapi-generator.tech
 */

use reqwest;

use super::{
    configuration,
    Error,
};
use crate::apis::ResponseContent;

/// struct for passing parameters to the method [`get_plugin_csi`]
#[derive(Clone, Debug, Default)]
pub struct GetPluginCsiParams {
    /// The CSI plugin identifier.
    pub plugin_id: String,
    /// Filters results based on the specified region.
    pub region: Option<String>,
    /// Filters results based on the specified namespace.
    pub namespace: Option<String>,
    /// If set, wait until query exceeds given index. Must be provided with WaitParam.
    pub index: Option<i32>,
    /// Provided with IndexParam to wait for change.
    pub wait: Option<String>,
    /// If present, results will include stale reads.
    pub stale: Option<String>,
    /// Constrains results to jobs that start with the defined prefix
    pub prefix: Option<String>,
    /// A Nomad ACL token.
    pub x_nomad_token: Option<String>,
    /// Maximum number of results to return.
    pub per_page: Option<i32>,
    /// Indicates where to start paging for queries that support pagination.
    pub next_token: Option<String>,
}

/// struct for passing parameters to the method [`get_plugins`]
#[derive(Clone, Debug, Default)]
pub struct GetPluginsParams {
    /// Filters results based on the specified region.
    pub region: Option<String>,
    /// Filters results based on the specified namespace.
    pub namespace: Option<String>,
    /// If set, wait until query exceeds given index. Must be provided with WaitParam.
    pub index: Option<i32>,
    /// Provided with IndexParam to wait for change.
    pub wait: Option<String>,
    /// If present, results will include stale reads.
    pub stale: Option<String>,
    /// Constrains results to jobs that start with the defined prefix
    pub prefix: Option<String>,
    /// A Nomad ACL token.
    pub x_nomad_token: Option<String>,
    /// Maximum number of results to return.
    pub per_page: Option<i32>,
    /// Indicates where to start paging for queries that support pagination.
    pub next_token: Option<String>,
}

/// struct for typed errors of method [`get_plugin_csi`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetPluginCsiError {
    Status400(),
    Status403(),
    Status405(),
    Status500(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_plugins`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetPluginsError {
    Status400(),
    Status403(),
    Status405(),
    Status500(),
    UnknownValue(serde_json::Value),
}

pub async fn get_plugin_csi(
    configuration: &configuration::Configuration,
    params: GetPluginCsiParams,
) -> Result<Vec<crate::models::CsiPlugin>, Error<GetPluginCsiError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let plugin_id = params.plugin_id;
    let region = params.region;
    let namespace = params.namespace;
    let index = params.index;
    let wait = params.wait;
    let stale = params.stale;
    let prefix = params.prefix;
    let x_nomad_token = params.x_nomad_token;
    let per_page = params.per_page;
    let next_token = params.next_token;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/plugin/csi/{pluginID}",
        local_var_configuration.base_path,
        pluginID = crate::apis::urlencode(plugin_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = region {
        local_var_req_builder =
            local_var_req_builder.query(&[("region", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = namespace {
        local_var_req_builder =
            local_var_req_builder.query(&[("namespace", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = wait {
        local_var_req_builder =
            local_var_req_builder.query(&[("wait", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = stale {
        local_var_req_builder =
            local_var_req_builder.query(&[("stale", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = prefix {
        local_var_req_builder =
            local_var_req_builder.query(&[("prefix", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = per_page {
        local_var_req_builder =
            local_var_req_builder.query(&[("per_page", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = next_token {
        local_var_req_builder =
            local_var_req_builder.query(&[("next_token", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(local_var_param_value) = index {
        local_var_req_builder =
            local_var_req_builder.header("index", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = x_nomad_token {
        local_var_req_builder =
            local_var_req_builder.header("X-Nomad-Token", local_var_param_value.to_string());
    }
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("X-Nomad-Token", local_var_value);
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetPluginCsiError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn get_plugins(
    configuration: &configuration::Configuration,
    params: GetPluginsParams,
) -> Result<Vec<crate::models::CsiPluginListStub>, Error<GetPluginsError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let region = params.region;
    let namespace = params.namespace;
    let index = params.index;
    let wait = params.wait;
    let stale = params.stale;
    let prefix = params.prefix;
    let x_nomad_token = params.x_nomad_token;
    let per_page = params.per_page;
    let next_token = params.next_token;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/plugins", local_var_configuration.base_path);
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = region {
        local_var_req_builder =
            local_var_req_builder.query(&[("region", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = namespace {
        local_var_req_builder =
            local_var_req_builder.query(&[("namespace", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = wait {
        local_var_req_builder =
            local_var_req_builder.query(&[("wait", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = stale {
        local_var_req_builder =
            local_var_req_builder.query(&[("stale", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = prefix {
        local_var_req_builder =
            local_var_req_builder.query(&[("prefix", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = per_page {
        local_var_req_builder =
            local_var_req_builder.query(&[("per_page", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = next_token {
        local_var_req_builder =
            local_var_req_builder.query(&[("next_token", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(local_var_param_value) = index {
        local_var_req_builder =
            local_var_req_builder.header("index", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = x_nomad_token {
        local_var_req_builder =
            local_var_req_builder.header("X-Nomad-Token", local_var_param_value.to_string());
    }
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("X-Nomad-Token", local_var_value);
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetPluginsError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}
