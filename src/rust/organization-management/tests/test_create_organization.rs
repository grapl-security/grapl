#![cfg(feature = "integration")]

use grapl_utils::future_ext::GraplFutureExt;
use organization_management::client::OrganizationManagementServiceClient;
use organization_management::OrganizationManagementServiceConfig;

use rust_proto::organization_management::{
    CreateOrganizationRequest,
    // CreateOrganizationResponse,
};

use structopt::StructOpt;

#[test_log::test(tokio::test)]
async fn test_create_organization() -> Result<(), Box<dyn std::error::Error>> {
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

    let (display_name, ): (String, ) = sqlx::query_as(
        r#"SELECT display_name
        FROM organization
        WHERE organization_id = $1
        "#,
    )
        .bind(organization_id)
        .fetch_one(&pool).await?;

    assert_eq!(display_name, organization_display_name);

    Ok(())
}

