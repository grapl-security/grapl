#![cfg(feature = "integration_tests")]

use bytes::Bytes;
use clap::Parser;
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::{
    client_factory::{
        build_grpc_client,
        services::PluginRegistryClientConfig,
    },
    graplinc::grapl::api::plugin_registry::v1beta1::{
        ListPluginsRequest,
        PluginMetadata,
        PluginType,
    },
};

fn get_example_generator() -> Result<Bytes, std::io::Error> {
    std::fs::read("/test-fixtures/example-generator").map(Bytes::from)
}

#[test_log::test(tokio::test)]
async fn test_list_plugins() -> eyre::Result<()> {
    let client_config = PluginRegistryClientConfig::parse();
    let mut client = build_grpc_client(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();
    let event_source_1_id = uuid::Uuid::new_v4();
    let event_source_2_id = uuid::Uuid::new_v4();
    let event_source_3_id = uuid::Uuid::new_v4();

    let generator_1_create_response = {
        let display_name = "generator 1";
        let artifact = get_example_generator()?;
        let metadata = PluginMetadata::new(
            tenant_id,
            display_name.to_string(),
            PluginType::Generator,
            Some(event_source_1_id.clone()),
        );

        client
            .create_plugin(
                metadata,
                futures::stream::once(async move { artifact.clone() }),
            )
            .timeout(std::time::Duration::from_secs(5))
            .await??
    };

    let generator_1_id = generator_1_create_response.plugin_id();

    let generator_2_create_response = {
        let display_name = "generator 2";
        let artifact = get_example_generator()?;
        let metadata = PluginMetadata::new(
            tenant_id,
            display_name.to_string(),
            PluginType::Generator,
            Some(event_source_2_id.clone()),
        );

        client
            .create_plugin(
                metadata,
                futures::stream::once(async move { artifact.clone() }),
            )
            .timeout(std::time::Duration::from_secs(5))
            .await??
    };

    let generator_2_id = generator_2_create_response.plugin_id();

    let analyzer_create_response = {
        let display_name = "analyzer";
        let artifact = get_example_generator()?;
        let metadata = PluginMetadata::new(
            tenant_id,
            display_name.to_string(),
            PluginType::Generator,
            Some(event_source_3_id.clone()),
        );

        client
            .create_plugin(
                metadata,
                futures::stream::once(async move { artifact.clone() }),
            )
            .timeout(std::time::Duration::from_secs(5))
            .await??
    };

    let analyzer_id = analyzer_create_response.plugin_id();

    let generator_plugins = client
        .list_plugins(ListPluginsRequest::new(tenant_id, PluginType::Generator))
        .timeout(std::time::Duration::from_secs(5))
        .await??
        .plugins();

    assert_eq!(generator_plugins.len(), 2);
    assert_eq!(generator_plugins[0].plugin_id(), generator_1_id);
    assert_eq!(generator_plugins[1].plugin_id(), generator_2_id);

    let analyzer_plugins = client
        .list_plugins(ListPluginsRequest::new(tenant_id, PluginType::Analyzer))
        .timeout(std::time::Duration::from_secs(5))
        .await??
        .plugins();

    assert_eq!(analyzer_plugins.len(), 1);
    assert_eq!(analyzer_plugins[0].plugin_id(), analyzer_id);

    Ok(())
}
