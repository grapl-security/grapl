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
pub struct PortMapping {
    #[serde(rename = "HostIP", skip_serializing_if = "Option::is_none")]
    pub host_ip: Option<String>,
    #[serde(rename = "Label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "To", skip_serializing_if = "Option::is_none")]
    pub to: Option<i32>,
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<i32>,
}

impl PortMapping {
    pub fn new() -> PortMapping {
        PortMapping {
            host_ip: None,
            label: None,
            to: None,
            value: None,
        }
    }
}
