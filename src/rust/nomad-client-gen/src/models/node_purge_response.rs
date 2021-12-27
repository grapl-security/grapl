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
pub struct NodePurgeResponse {
    #[serde(rename = "EvalCreateIndex", skip_serializing_if = "Option::is_none")]
    pub eval_create_index: Option<i32>,
    #[serde(rename = "EvalIDs", skip_serializing_if = "Option::is_none")]
    pub eval_ids: Option<Vec<String>>,
    #[serde(rename = "NodeModifyIndex", skip_serializing_if = "Option::is_none")]
    pub node_modify_index: Option<i32>,
}

impl NodePurgeResponse {
    pub fn new() -> NodePurgeResponse {
        NodePurgeResponse {
            eval_create_index: None,
            eval_ids: None,
            node_modify_index: None,
        }
    }
}


