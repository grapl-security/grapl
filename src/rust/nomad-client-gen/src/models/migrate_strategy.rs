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
pub struct MigrateStrategy {
    #[serde(rename = "HealthCheck", skip_serializing_if = "Option::is_none")]
    pub health_check: Option<String>,
    #[serde(rename = "HealthyDeadline", skip_serializing_if = "Option::is_none")]
    pub healthy_deadline: Option<i64>,
    #[serde(rename = "MaxParallel", skip_serializing_if = "Option::is_none")]
    pub max_parallel: Option<i32>,
    #[serde(rename = "MinHealthyTime", skip_serializing_if = "Option::is_none")]
    pub min_healthy_time: Option<i64>,
}

impl MigrateStrategy {
    pub fn new() -> MigrateStrategy {
        MigrateStrategy {
            health_check: None,
            healthy_deadline: None,
            max_parallel: None,
            min_healthy_time: None,
        }
    }
}
