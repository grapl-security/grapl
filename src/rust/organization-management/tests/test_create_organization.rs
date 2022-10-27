#![cfg(feature = "integration_tests")]

use clap::Parser;
use figment::{
    providers::Env,
    Figment,
};
use grapl_config::ToPostgresUrl;
use organization_management::OrganizationManagementServiceConfig;
use rust_proto::graplinc::grapl::api::{
    client::Connect,
    organization_management::v1beta1::{
        client::OrganizationManagementClient,
        CreateOrganizationRequest,
    },
};

#[test_log::test(tokio::test)]
async fn test_create_organization() -> eyre::Result<()> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let service_config = OrganizationManagementServiceConfig::parse();

    let pool = service_config.to_postgres_url().connect().await?;

    let client_config = Figment::new()
        .merge(Env::prefixed("ORGANIZATION_MANAGEMENT_CLIENT_"))
        .extract()?;
    let mut client = OrganizationManagementClient::connect(client_config).await?;

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
