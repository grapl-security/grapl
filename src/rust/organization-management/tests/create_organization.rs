#![cfg(feature = "integration")]

use grapl_utils::future_ext::GraplFutureExt;
use organization_management::client::OrganizationManagementServiceClient;
use rust_proto::organization_management::{
    CreateOrganizationRequest,
};


#[test_log::test(tokio::test)]
async fn test_create_organization() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let mut client = OrganizationManagementServiceClient::from_env().await?;

    let test_display_name = uuid::Uuid::new_v4().to_string();
    let test_admin_username = "test user".to_string();
    let test_admin_email = "testemail@email.com";
    let test_admin_password = b"t3stp@s$w0rd".to_vec();

    let request = CreateOrganizationRequest {
        organization_display_name: "Display Name Inc.".parse().unwrap(),
        admin_username: test_admin_username,
        admin_email: test_user_email.parse().unwrap(),
        admin_password: test_admin_password,
        should_reset_password: false,
    };

    let response = client
        .create_organization(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let get_response: GetOrganizationResponse = client
        .get_organization(GetOrganizationRequest {
            organization_display_name,
            admin_username,
            admin_email,
            admin_password,
            organization_id,
        })
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    assert_eq!(get_response.organization.organization_display_name, organization_display_name);
    assert_eq!(get_response.organization.admin_username, admin_usernam);
    assert_eq!(get_response.organization.admin_email, admin_email);
    assert_eq!(get_response.organization.admin_password, admin_password);
    assert_eq!(get_response.organization.should_reset_password, should_reset_password);

    Ok(())
}
