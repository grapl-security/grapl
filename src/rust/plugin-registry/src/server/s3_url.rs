use grapl_config::env_helpers::ENV_ENDPOINT;

/// Some discussion about this function here:
/// https://grapl-internal.slack.com/archives/C02J5JYS92S/p1648667074625629
pub fn get_s3_url(bucket: &str, key: &str) -> String {
    let local_endpoint = std::env::var(ENV_ENDPOINT).ok();
    // If the above is specified, we're running locally against Localhost
    // Otherwise, it's against prod S3; so we can use virtual-hosted-style URLs
    local_endpoint.map_or_else(
        || format!("http://{bucket}.s3.amazonaws.com/{key}"),
        |endpoint| format!("{endpoint}/{bucket}/{key}"),
    )
}
