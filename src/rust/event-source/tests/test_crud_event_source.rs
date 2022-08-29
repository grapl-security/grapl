#![cfg(feature = "integration_tests")]

use std::time::SystemTime;

use clap::Parser;
use rust_proto::{
    client_factory::{
        build_grpc_client,
        services::EventSourceClientConfig,
    },
    graplinc::grapl::api::event_source::v1beta1 as es_api,
};

#[test_log::test(tokio::test)]
async fn test_create_update_get() -> eyre::Result<()> {
    let client_config = EventSourceClientConfig::parse();
    let mut client = build_grpc_client(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();

    // Create an event source
    let create_response = {
        let request = es_api::CreateEventSourceRequest {
            display_name: "Name v1".to_owned(),
            description: "Description v1".to_owned(),
            tenant_id,
        };
        client.create_event_source(request).await?
    };

    assert!(create_response.created_time <= SystemTime::now());

    // Ensure it matches default expectations
    let get_response = {
        let request = es_api::GetEventSourceRequest {
            event_source_id: create_response.event_source_id,
        };
        client.get_event_source(request).await?
    };

    assert!(get_response.event_source.display_name == "Name v1");
    assert!(get_response.event_source.description == "Description v1");
    assert!(get_response.event_source.created_time == create_response.created_time);
    assert!(get_response.event_source.active);

    // Do an update on all modifiable fields
    let update_response = {
        let request = es_api::UpdateEventSourceRequest {
            event_source_id: get_response.event_source.event_source_id,
            display_name: "Name v2".to_owned(),
            description: "Description v2".to_owned(),
            active: false,
        };
        client.update_event_source(request).await?
    };

    // Ensure the update time has changed
    assert!(update_response.last_updated_time > get_response.event_source.last_updated_time);

    // Get it again
    let get_response = {
        let request = es_api::GetEventSourceRequest {
            event_source_id: create_response.event_source_id,
        };
        client.get_event_source(request).await?
    };

    assert!(get_response.event_source.display_name == "Name v2");
    assert!(get_response.event_source.description == "Description v2");
    assert!(!get_response.event_source.active);

    Ok(())
}
