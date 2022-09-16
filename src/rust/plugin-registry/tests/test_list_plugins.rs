#![cfg(feature = "integration_tests")]

use bytes::Bytes;
use clap::Parser;
use rust_proto::graplinc::grapl::api::{
    client_factory::services::PluginRegistryClientConfig,
    plugin_registry::v1beta1::{
        ListPluginsRequest,
        PluginMetadata,
        PluginRegistryServiceClient,
        PluginType,
    },
    protocol::service_client::ConnectWithConfig,
};

#[test_log::test(tokio::test)]
async fn test_list_plugins() -> eyre::Result<()> {
    let client_config = PluginRegistryClientConfig::parse();
    let mut client = PluginRegistryServiceClient::connect_with_config(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();
    let event_source_1_id = uuid::Uuid::new_v4();
    let event_source_2_id = uuid::Uuid::new_v4();

    let generator_1_create_response = {
        let display_name = "generator 1";
        let artifact = Bytes::from("fake");
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
            .await?
    };

    let generator_1_id = generator_1_create_response.plugin_id();

    let generator_2_create_response = {
        let display_name = "generator 2";
        let artifact = Bytes::from("fake");
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
            .await?
    };

    let generator_2_id = generator_2_create_response.plugin_id();

    let analyzer_create_response = {
        let display_name = "analyzer";
        let artifact = Bytes::from("fake");
        let metadata = PluginMetadata::new(
            tenant_id,
            display_name.to_string(),
            PluginType::Analyzer,
            None,
        );

        client
            .create_plugin(
                metadata,
                futures::stream::once(async move { artifact.clone() }),
            )
            .await?
    };

    let analyzer_id = analyzer_create_response.plugin_id();

    let generator_plugins = client
        .list_plugins(ListPluginsRequest::new(tenant_id, PluginType::Generator))
        .await?
        .plugins();

    assert_eq!(generator_plugins.len(), 2);
    assert_eq!(generator_plugins[0].plugin_id(), generator_1_id);
    assert_eq!(generator_plugins[1].plugin_id(), generator_2_id);

    let analyzer_plugins = client
        .list_plugins(ListPluginsRequest::new(tenant_id, PluginType::Analyzer))
        .await?
        .plugins();

    assert_eq!(analyzer_plugins.len(), 1);
    assert_eq!(analyzer_plugins[0].plugin_id(), analyzer_id);

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_list_plugins_not_found() -> eyre::Result<()> {
    let client_config = PluginRegistryClientConfig::parse();
    let mut client = PluginRegistryServiceClient::connect_with_config(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();

    let generator_plugins = client
        .list_plugins(ListPluginsRequest::new(tenant_id, PluginType::Generator))
        .await?;

    assert_eq!(generator_plugins.plugins().len(), 0);

    Ok(())
}
