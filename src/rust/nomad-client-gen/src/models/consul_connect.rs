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
pub struct ConsulConnect {
    #[serde(rename = "Gateway", skip_serializing_if = "Option::is_none")]
    pub gateway: Option<Box<crate::models::ConsulGateway>>,
    #[serde(rename = "Native", skip_serializing_if = "Option::is_none")]
    pub native: Option<bool>,
    #[serde(rename = "SidecarService", skip_serializing_if = "Option::is_none")]
    pub sidecar_service: Option<Box<crate::models::ConsulSidecarService>>,
    #[serde(rename = "SidecarTask", skip_serializing_if = "Option::is_none")]
    pub sidecar_task: Option<Box<crate::models::SidecarTask>>,
}

impl ConsulConnect {
    pub fn new() -> ConsulConnect {
        ConsulConnect {
            gateway: None,
            native: None,
            sidecar_service: None,
            sidecar_task: None,
        }
    }
}
