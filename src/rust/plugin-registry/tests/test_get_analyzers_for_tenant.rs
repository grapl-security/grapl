#![cfg(feature = "integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use figment::{
    providers::Env,
    Figment,
};
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::{
    client::{
        ClientError,
        Connect,
    },
    plugin_registry::v1beta1::{
        GetAnalyzersForTenantRequest,
        PluginMetadata,
        PluginRegistryClient,
        PluginType,
    },
    protocol::status::Code,
};

#[test_log::test(tokio::test)]
async fn test_get_analyzers_for_tenant() -> eyre::Result<()> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let client_config = Figment::new()
        .merge(Env::prefixed("PLUGIN_REGISTRY_CLIENT_"))
        .extract()?;
    let mut client = PluginRegistryClient::connect(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();
    let analyzer1_display_name = "my first analyzer".to_string();
    let analyzer2_display_name = "my second analyzer".to_string();

    let generator_display_name = "my generator".to_string();
    let event_source_id = uuid::Uuid::new_v4();

    let analyzer1_metadata = PluginMetadata::new(
        tenant_id,
        analyzer1_display_name.clone(),
        PluginType::Analyzer,
        None,
    );

    let analyzer2_metadata = PluginMetadata::new(
        tenant_id,
        analyzer2_display_name.clone(),
        PluginType::Analyzer,
        None,
    );

    let generator_metadata = PluginMetadata::new(
        tenant_id,
        generator_display_name.clone(),
        PluginType::Generator,
        Some(event_source_id),
    );

    let chunk = Bytes::from("chonk");

    let create_analyzer1_chunk = chunk.clone();
    let create_analyzer1_response = client
        .create_plugin(
            analyzer1_metadata,
            futures::stream::once(async move { create_analyzer1_chunk }),
        )
        .timeout(Duration::from_secs(5))
        .await??;
    let analyzer1_plugin_id = create_analyzer1_response.plugin_id();

    let create_analyzer2_chunk = chunk.clone();
    let create_analyzer2_response = client
        .create_plugin(
            analyzer2_metadata,
            futures::stream::once(async move { create_analyzer2_chunk }),
        )
        .timeout(Duration::from_secs(5))
        .await??;
    let analyzer2_plugin_id = create_analyzer2_response.plugin_id();

    let create_generator_chunk = chunk.clone();
    let create_generator_response = client
        .create_plugin(
            generator_metadata,
            futures::stream::once(async move { create_generator_chunk }),
        )
        .timeout(Duration::from_secs(5))
        .await??;
    let generator_plugin_id = create_generator_response.plugin_id();

    let analyzers_for_tenant_response = client
        .get_analyzers_for_tenant(GetAnalyzersForTenantRequest::new(tenant_id))
        .timeout(Duration::from_secs(5))
        .await??;

    assert_eq!(analyzers_for_tenant_response.plugin_ids().len(), 2);
    assert!(analyzers_for_tenant_response
        .plugin_ids()
        .contains(&analyzer1_plugin_id));
    assert!(analyzers_for_tenant_response
        .plugin_ids()
        .contains(&analyzer2_plugin_id));
    assert!(!analyzers_for_tenant_response
        .plugin_ids()
        .contains(&generator_plugin_id));

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_get_analyzers_for_tenant_not_found() -> eyre::Result<()> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let client_config = Figment::new()
        .merge(Env::prefixed("PLUGIN_REGISTRY_CLIENT_"))
        .extract()?;
    let mut client = PluginRegistryClient::connect(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();

    if let Err(e) = client
        .get_analyzers_for_tenant(GetAnalyzersForTenantRequest::new(tenant_id))
        .timeout(Duration::from_secs(5))
        .await?
    {
        match e {
            ClientError::Status(s) => {
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
