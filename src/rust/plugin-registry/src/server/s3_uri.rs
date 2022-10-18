use grapl_config::env_helpers::ENV_ENDPOINT;

/// This gets a s3 URI to be used by Nomad's artifact stanza (in particular the grapl-plugin).
/// The underlying library, Hashicorp's [go-getter](https://github.com/hashicorp/go-getter) lib,
/// doesn't work with the `http://` schema prefix, but does work with an explicit s3 prefix `s://`
/// or without a prefix. We've opted for the s3 prefix to be explicit.
pub fn get_s3_uri(bucket: &str, key: &str) -> String {
    let local_endpoint = std::env::var(ENV_ENDPOINT).ok();
    // If the above is specified, we're running locally against Localhost
    // Otherwise, it's against prod S3; so we use the s3 URIs. Notably, the underlying go-getter
    // lib does not seem to work with s3 object URLs (urls that start with http:// or https://),
    // including the recommended virtual host style access.
    // https://github.com/hashicorp/go-getter/issues/387 tracks updating the documentation.
    // This is the s3 URI format used in the s3 console.
    local_endpoint.map_or_else(
        || format!("s3://{bucket}.s3.amazonaws.com/{key}"),
        |endpoint| format!("{endpoint}/{bucket}/{key}"),
    )
}
