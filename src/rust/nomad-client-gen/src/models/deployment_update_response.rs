/*
 * Nomad
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.1.4
 * Contact: support@hashicorp.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct DeploymentUpdateResponse {
    #[serde(
        rename = "DeploymentModifyIndex",
        skip_serializing_if = "Option::is_none"
    )]
    pub deployment_modify_index: Option<i32>,
    #[serde(rename = "EvalCreateIndex", skip_serializing_if = "Option::is_none")]
    pub eval_create_index: Option<i32>,
    #[serde(rename = "EvalID", skip_serializing_if = "Option::is_none")]
    pub eval_id: Option<String>,
    #[serde(rename = "LastIndex", skip_serializing_if = "Option::is_none")]
    pub last_index: Option<i32>,
    #[serde(rename = "RequestTime", skip_serializing_if = "Option::is_none")]
    pub request_time: Option<i64>,
    #[serde(rename = "RevertedJobVersion", skip_serializing_if = "Option::is_none")]
    pub reverted_job_version: Option<i32>,
}

impl DeploymentUpdateResponse {
    pub fn new() -> DeploymentUpdateResponse {
        DeploymentUpdateResponse {
            deployment_modify_index: None,
            eval_create_index: None,
            eval_id: None,
            last_index: None,
            request_time: None,
            reverted_job_version: None,
        }
    }
}
