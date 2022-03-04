#![cfg(feature = "integration")]

use grapl_utils::future_ext::GraplFutureExt;
use organization_management::{
    client::OrganizationManagementServiceClient,
    OrganizationManagementServiceConfig,
};
use rust_proto::organization_management::{
    CreateOrganizationRequest,
    CreateUserRequest,
};
use structopt::StructOpt;

#[test_log::test(tokio::test)]
async fn test_create_user() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let service_config = OrganizationManagementServiceConfig::from_args();

    let postgres_address = format!(
        "postgresql://{}:{}@{}:{}",
        service_config.organization_management_db_username,
        service_config.organization_management_db_password,
        service_config.organization_management_db_hostname,
        service_config.organization_management_db_port,
    );

    let pool = sqlx::PgPool::connect(&postgres_address)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let mut client = OrganizationManagementServiceClient::from_env().await?;
    // create organization with admin user
    let organization_display_name = uuid::Uuid::new_v4().to_string();
    let admin_username = "test user".to_string();
    let admin_email = "testemail@email.com";
    let admin_password = b"t3stp@s$w0rd".to_vec();

    let request = CreateOrganizationRequest {
        organization_display_name: organization_display_name.clone(),
        admin_username,
        admin_email: admin_email.parse().unwrap(),
        admin_password,
        should_reset_password: false,
    };

    let response = client
        .create_organization(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let organization_id = response.organization_id;

    let name = "user test".to_string();
    let email = "testinguseremail@example.com";
    let password = b"t3stp@s$w0rd!".to_vec();

    // create user for organization that already has an admin
    let request = CreateUserRequest {
        organization_id,
        name,
        email: email.parse().unwrap(),
        password,
    };

    let response = client
        .create_user(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

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
