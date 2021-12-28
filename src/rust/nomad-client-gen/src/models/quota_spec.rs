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
pub struct QuotaSpec {
    #[serde(rename = "CreateIndex", skip_serializing_if = "Option::is_none")]
    pub create_index: Option<i32>,
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "Limits", skip_serializing_if = "Option::is_none")]
    pub limits: Option<Vec<crate::models::QuotaLimit>>,
    #[serde(rename = "ModifyIndex", skip_serializing_if = "Option::is_none")]
    pub modify_index: Option<i32>,
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl QuotaSpec {
    pub fn new() -> QuotaSpec {
        QuotaSpec {
            create_index: None,
            description: None,
            limits: None,
            modify_index: None,
            name: None,
        }
    }
}
