#![cfg(feature = "new_integration_tests")]

use std::time::Duration;

use clap::Parser;
use grapl_utils::future_ext::GraplFutureExt;
use organization_management::OrganizationManagementServiceConfig;
use rust_proto::{
    graplinc::grapl::api::organization_management::v1beta1::{
        client::OrganizationManagementClient,
        CreateOrganizationRequest,
    },
    protocol::healthcheck::client::HealthcheckClient,
};

#[test_log::test(tokio::test)]
async fn test_create_organization() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let service_config = OrganizationManagementServiceConfig::parse();

    let postgres_address = format!(
        "postgresql://{}:{}@{}:{}",
        service_config.organization_management_db_username,
        service_config.organization_management_db_password,
        service_config.organization_management_db_hostname,
        service_config.organization_management_db_port,
    );

    let pool = sqlx::PgPool::connect(&postgres_address)
        .timeout(Duration::from_secs(5))
        .await??;

    let endpoint = std::env::var("ORGANIZATION_MANAGEMENT_CLIENT_ADDRESS")
        .expect("missing environment variable ORGANIZATION_MANAGEMENT_CLIENT_ADDRESS");

    tracing::info!(
        message = "waiting 60s for organization-management to report healthy",
        endpoint = %endpoint,
    );

    HealthcheckClient::wait_until_healthy(
        endpoint.clone(),
        "graplinc.grapl.api.organization_management.v1beta1.OrganizationManagementService",
        Duration::from_secs(60),
        Duration::from_millis(500),
    )
    .await
    .expect("organization-management never reported healthy");

    let mut client = OrganizationManagementClient::connect(endpoint.clone()).await?;

    let organization_display_name = uuid::Uuid::new_v4().to_string();
    let admin_username = "test user".to_string();
    let admin_email = "testemail@example.com";
    let admin_password = b"t3stp@s$w0rd".to_vec();

    let request = CreateOrganizationRequest {
        organization_display_name: organization_display_name.clone(),
        admin_username,
        admin_email: admin_email.parse().unwrap(),
        admin_password: admin_password.into(),
        should_reset_password: false,
    };

    let response = client.create_organization(request).await?;

    let organization_id = response.organization_id;

    let (display_name,): (String,) = sqlx::query_as(
        r#"SELECT display_name
        FROM organizations
        WHERE organization_id = $1
        "#,
    )
    .bind(organization_id)
    .fetch_one(&pool)
    .await?;

    assert_eq!(display_name, organization_display_name);

    Ok(())
}
