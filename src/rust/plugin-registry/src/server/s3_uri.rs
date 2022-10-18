use grapl_config::env_helpers::ENV_ENDPOINT;

/// This gets a s3 URI to be used by Nomad's artifact stanza (in particular the grapl-plugin).
/// The underlying go-getter lib, doesn't work with the `http://` schema prefix, but does work with
/// an explicit s3 prefix `s::` or without a prefix. We've opted for the s3 prefix to be explicit.
pub fn get_s3_uri(bucket: &str, key: &str) -> String {
    let local_endpoint = std::env::var(ENV_ENDPOINT).ok();
    // If the above is specified, we're running locally against Localhost
    // Otherwise, it's against prod S3; so we can use virtual-hosted-style URLs
    local_endpoint.map_or_else(
        || format!("s3::{bucket}.s3.amazonaws.com/{key}"),
        |endpoint| format!("{endpoint}/{bucket}/{key}"),
    )
}
