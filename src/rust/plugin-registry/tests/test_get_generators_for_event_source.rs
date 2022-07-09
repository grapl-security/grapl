#![cfg(feature = "integration_tests")]

use bytes::Bytes;
use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::FromEnv;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetGeneratorsForEventSourceRequest,
    PluginMetadata,
    PluginRegistryServiceClient,
    PluginType,
};

#[test_log::test(tokio::test)]
async fn test_get_generators_for_event_source() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let mut client = PluginRegistryServiceClient::from_env().await?;

    let tenant_id = uuid::Uuid::new_v4();
    let generator1_display_name = "my first generator".to_string();
    let generator2_display_name = "my second generator".to_string();
    let analyzer_display_name = "my analyzer".to_string();
    let event_source_id = uuid::Uuid::new_v4();

    let generator1_metadata = PluginMetadata {
        tenant_id: tenant_id.clone(),
        display_name: generator1_display_name.clone(),
        plugin_type: PluginType::Generator,
        event_source_id: Some(event_source_id),
    };
    let generator2_metadata = PluginMetadata {
        tenant_id: tenant_id.clone(),
        display_name: generator2_display_name.clone(),
        plugin_type: PluginType::Generator,
        event_source_id: Some(event_source_id),
    };

    let analyzer_metadata = PluginMetadata {
        tenant_id: tenant_id.clone(),
        display_name: analyzer_display_name.clone(),
        plugin_type: PluginType::Analyzer,
        event_source_id: None,
    };

    let chunk = Bytes::from("chonk");

    let create_generator1_chunk = chunk.clone();
    let create_generator1_response = client
        .create_plugin(
            generator1_metadata,
            futures::stream::once(async move { create_generator1_chunk }),
        )
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    let generator1_plugin_id = create_generator1_response.plugin_id;

    let create_generator2_chunk = chunk.clone();
    let create_generator2_response = client
        .create_plugin(
            generator2_metadata,
            futures::stream::once(async move { create_generator2_chunk }),
        )
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    let generator2_plugin_id = create_generator2_response.plugin_id;

    let create_analyzer_chunk = chunk.clone();
    let create_analyzer_response = client
        .create_plugin(
            analyzer_metadata,
            futures::stream::once(async move { create_analyzer_chunk }),
        )
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    let analyzer_plugin_id = create_analyzer_response.plugin_id;

    let generators_for_event_source_response = client
        .get_generators_for_event_source(GetGeneratorsForEventSourceRequest { event_source_id })
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    assert_eq!(generators_for_event_source_response.plugin_ids.len(), 2);
    assert!(generators_for_event_source_response
        .plugin_ids
        .contains(&generator1_plugin_id));
    assert!(generators_for_event_source_response
        .plugin_ids
        .contains(&generator2_plugin_id));
    assert!(!generators_for_event_source_response
        .plugin_ids
        .contains(&analyzer_plugin_id));

    Ok(())
}
