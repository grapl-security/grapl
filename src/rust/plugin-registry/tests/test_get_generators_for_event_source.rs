#![cfg(feature = "integration_tests")]

use bytes::Bytes;
use clap::Parser;
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::{
    client_factory::services::PluginRegistryClientConfig,
    plugin_registry::v1beta1::{
        GetGeneratorsForEventSourceRequest,
        PluginMetadata,
        PluginRegistryServiceClient,
        PluginType,
    },
    protocol::{
        error::GrpcClientError,
        service_client::ConnectWithConfig,
        status::Code,
    },
};

#[test_log::test(tokio::test)]
async fn test_get_generators_for_event_source() -> eyre::Result<()> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let client_config = PluginRegistryClientConfig::parse();
    let mut client = PluginRegistryServiceClient::connect_with_config(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();
    let generator1_display_name = "my first generator".to_string();
    let generator2_display_name = "my second generator".to_string();
    let analyzer_display_name = "my analyzer".to_string();
    let event_source_id = uuid::Uuid::new_v4();

    let generator1_metadata = PluginMetadata::new(
        tenant_id,
        generator1_display_name.clone(),
        PluginType::Generator,
        Some(event_source_id),
    );

    let generator2_metadata = PluginMetadata::new(
        tenant_id,
        generator2_display_name.clone(),
        PluginType::Generator,
        Some(event_source_id),
    );

    let analyzer_metadata = PluginMetadata::new(
        tenant_id,
        analyzer_display_name.clone(),
        PluginType::Analyzer,
        None,
    );

    let chunk = Bytes::from("chonk");

    let create_generator1_chunk = chunk.clone();
    let create_generator1_response = client
        .create_plugin(
            generator1_metadata,
            futures::stream::once(async move { create_generator1_chunk }),
        )
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    let generator1_plugin_id = create_generator1_response.plugin_id();

    let create_generator2_chunk = chunk.clone();
    let create_generator2_response = client
        .create_plugin(
            generator2_metadata,
            futures::stream::once(async move { create_generator2_chunk }),
        )
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    let generator2_plugin_id = create_generator2_response.plugin_id();

    let create_analyzer_chunk = chunk.clone();
    let create_analyzer_response = client
        .create_plugin(
            analyzer_metadata,
            futures::stream::once(async move { create_analyzer_chunk }),
        )
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    let analyzer_plugin_id = create_analyzer_response.plugin_id();

    let generators_for_event_source_response = client
        .get_generators_for_event_source(GetGeneratorsForEventSourceRequest::new(event_source_id))
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    assert_eq!(generators_for_event_source_response.plugin_ids().len(), 2);
    assert!(generators_for_event_source_response
        .plugin_ids()
        .contains(&generator1_plugin_id));
    assert!(generators_for_event_source_response
        .plugin_ids()
        .contains(&generator2_plugin_id));
    assert!(!generators_for_event_source_response
        .plugin_ids()
        .contains(&analyzer_plugin_id));

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_get_generators_for_event_source_not_found() -> eyre::Result<()> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let client_config = PluginRegistryClientConfig::parse();
    let mut client = PluginRegistryServiceClient::connect_with_config(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();

    if let Err(e) = client
        .get_generators_for_event_source(GetGeneratorsForEventSourceRequest::new(tenant_id))
        .timeout(std::time::Duration::from_secs(5))
        .await?
    {
        match e {
            GrpcClientError::ErrorStatus(s) => {
                if let Code::NotFound = s.code() {
                    Ok(()) // 👍 great success 👍
                } else {
                    panic!("unexpected status code {}", s.code())
                }
            }
            e => panic!("unexpected error {}", e),
        }
    } else {
        panic!("expected error")
    }
}
