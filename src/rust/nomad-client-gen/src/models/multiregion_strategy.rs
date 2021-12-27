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
pub struct MultiregionStrategy {
    #[serde(rename = "MaxParallel", skip_serializing_if = "Option::is_none")]
    pub max_parallel: Option<i32>,
    #[serde(rename = "OnFailure", skip_serializing_if = "Option::is_none")]
    pub on_failure: Option<String>,
}

impl MultiregionStrategy {
    pub fn new() -> MultiregionStrategy {
        MultiregionStrategy {
            max_parallel: None,
            on_failure: None,
        }
    }
}


