#![cfg(feature = "integration")]

use grapl_utils::future_ext::GraplFutureExt;
use organization_management::client::OrganizationManagementServiceClient;
use rust_proto::organization_management::{
    CreateUserRequest,
};


#[test_log::test(tokio::test)]
async fn test_create_user() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(
        env=?std::env::args(),
    );

    let mut client = OrganizationManagementServiceClient::from_env().await?;

    let test_name = uuid::Uuid::new_v4().to_string();
    let test_email = "testemail@email.com";
    let test_password = b"t3stp@s$w0rd".to_vec();

    let request = CreateUserRequest {
        organization_id: Default::default(),
        name: test_name,
        email: test_user_email.parse().unwrap(),
        password: test_admin_password,
    };

    let response = client
        .create_user(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let get_response: GetUserResponse = client
        .get_user(GetUserRequest {
            name,
            email,
            password,
        })
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    assert_eq!(get_response.organization.name, name);
    assert_eq!(get_response.organization.email, email);
    assert_eq!(get_response.organization.password, password);

    Ok(())
}
