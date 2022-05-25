#![cfg(feature = "new_integration_tests")]

use std::time::Duration;

use futures_retry::{
    FutureRetry,
    RetryPolicy,
};
use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::FromEnv;
use rust_proto_new::graplinc::grapl::api::plugin_registry::v1beta1::{
    CreatePluginRequestMetadata,
    DeployPluginRequest,
    PluginRegistryServiceClient,
    PluginRegistryServiceClientError,
    PluginType,
};

pub const SMALL_TEST_BINARY: &'static [u8] = include_bytes!("./small_test_binary.sh");

fn get_example_generator() -> Result<Vec<u8>, std::io::Error> {
    std::fs::read("/test-fixtures/example-generator")
}

pub struct GetClientErrorHandler {}

impl futures_retry::ErrorHandler<PluginRegistryServiceClientError> for GetClientErrorHandler {
    type OutError = PluginRegistryServiceClientError;

    fn handle(
        &mut self,
        attempt: usize,
        e: PluginRegistryServiceClientError,
    ) -> RetryPolicy<Self::OutError> {
        let attempt = attempt as u64;
        match attempt {
            0..=2 => RetryPolicy::WaitRetry(Duration::from_secs(1)),
            _ => RetryPolicy::ForwardError(e),
        }
    }
}

async fn get_client() -> Result<PluginRegistryServiceClient, PluginRegistryServiceClientError> {
    // For some reason, I'm seeing nondeterministic failures when initializing
    // a client.
    // Suspicions:
    // - it's due to creating two clients at exactly the same time
    //   (these two async tests are concurrent)
    // - maybe something about warming up a connection pool?
    // Anyway, I have no evidence, but this seems to do the trick. Weird.
    let (response, _) = FutureRetry::new(
        PluginRegistryServiceClient::from_env,
        GetClientErrorHandler {},
    )
    .await
    .map_err(|(err, _)| err)?;
    Ok(response)
}

#[test_log::test(tokio::test)]
async fn test_deploy_plugin() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_client().await?;

    let tenant_id = uuid::Uuid::new_v4();

    let create_response = {
        let display_name = uuid::Uuid::new_v4().to_string();
        let artifact = get_example_generator()?;
        let metadata = CreatePluginRequestMetadata {
            tenant_id: tenant_id.clone(),
            display_name: display_name.clone(),
            plugin_type: PluginType::Generator,
        };

        client
            .create_plugin(metadata, artifact.into_iter())
            .timeout(std::time::Duration::from_secs(5))
            .await??
    };

    let plugin_id = create_response.plugin_id;

    let request = DeployPluginRequest { plugin_id };

    let _response = client
        .deploy_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    Ok(())
}

fn assert_contains(input: &str, expected_substr: &str) {
    assert!(
        input.contains(expected_substr),
        "Expected input '{input}' to contain '{expected_substr}'"
    )
}

#[test_log::test(tokio::test)]
/// So we *expect* this call to fail since it's an arbitrary PluginID that
/// hasn't been created yet
async fn test_deploy_plugin_but_plugin_id_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_client().await?;

    let randomly_selected_plugin_id = uuid::Uuid::new_v4();

    let request = DeployPluginRequest {
        plugin_id: randomly_selected_plugin_id,
    };

    let response = client
        .deploy_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await?;

    match response {
        Err(PluginRegistryServiceClientError::ErrorStatus(s)) => {
            // TODO: We should consider a dedicated "PluginIDDoesntExist" exception
            assert_contains(s.message(), "Failed to operate on postgres");
        }
        _ => panic!("Expected an error"),
    };
    Ok(())
}
