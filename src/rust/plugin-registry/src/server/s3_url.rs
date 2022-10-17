use grapl_config::env_helpers::ENV_ENDPOINT;

pub fn get_s3_url(bucket: &str, key: &str) -> String {
    let local_endpoint = std::env::var(ENV_ENDPOINT).ok();
    // If the above is specified, we're running locally against Localhost
    // Otherwise, it's against prod S3; so we can use virtual-hosted-style URLs
    local_endpoint.map_or_else(
        || format!("s3::{bucket}.s3.amazonaws.com/{key}"),
        |endpoint| format!("{endpoint}/{bucket}/{key}"),
    )
}
