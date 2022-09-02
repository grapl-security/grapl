#![cfg(feature = "integration_tests")]

use clap::Parser;
use grapl_config::ToPostgresUrl;
use organization_management::OrganizationManagementServiceConfig;
use rust_proto::{
    client_factory::{
        build_grpc_client,
        services::OrganizationManagementClientConfig,
    },
    graplinc::grapl::api::organization_management::v1beta1::{
        CreateOrganizationRequest,
        CreateUserRequest,
    },
};

#[test_log::test(tokio::test)]
async fn test_create_user() -> eyre::Result<()> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let service_config = OrganizationManagementServiceConfig::parse();

    let pool = service_config.to_postgres_url().connect().await?;

    let client_config = OrganizationManagementClientConfig::parse();
    let mut client = build_grpc_client(client_config).await?;

    // create organization with admin user
    let organization_display_name = uuid::Uuid::new_v4().to_string();
    let admin_username = "test user".to_string();
    let admin_email = "testemail@email.com";
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

    let name = "user test".to_string();
    let email = "testinguseremail@example.com";
    let password = b"t3stp@s$w0rd!".to_vec();

    // create user for organization that already has an admin
    let request = CreateUserRequest {
        organization_id,
        name,
        email: email.parse().unwrap(),
        password: password.into(),
    };

    let response = client.create_user(request).await?;

    let user_id = response.user_id;

    let (name, email): (String, String) = sqlx::query_as(
        r#"SELECT username, email
        FROM users
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

    assert_eq!(name, "user test");
    assert_eq!(email, "testinguseremail@example.com");

    Ok(())
}
