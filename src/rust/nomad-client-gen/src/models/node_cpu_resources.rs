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
pub struct NodeCpuResources {
    #[serde(rename = "CpuShares", skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<i64>,
    #[serde(rename = "ReservableCpuCores", skip_serializing_if = "Option::is_none")]
    pub reservable_cpu_cores: Option<Vec<i32>>,
    #[serde(rename = "TotalCpuCores", skip_serializing_if = "Option::is_none")]
    pub total_cpu_cores: Option<i32>,
}

impl NodeCpuResources {
    pub fn new() -> NodeCpuResources {
        NodeCpuResources {
            cpu_shares: None,
            reservable_cpu_cores: None,
            total_cpu_cores: None,
        }
    }
}
