// #![cfg(feature = "integration")]
//
// use organization_management::client::OrganizationManagementServiceClient;
//
// use rust_proto::organization_management::{
//     CreateUserRequest,
//     CreateUserResponse
// };
//
//
// #[test_log::test(tokio::test)]
// async fn test_create_user() -> Result<(), Box<dyn std::error::Error>> {
//     tracing::debug!(
//         env=?std::env::args(),
//     );
//
//     let mut client = OrganizationManagementServiceClient::new(OrganizationManagementServiceClient);
//
//     let test_name = uuid::Uuid::new_v4().to_string();
//     let test_email = "testemail@email.com";
//     let test_password = b"t3stp@s$w0rd".to_vec();
//
//     let request = CreateUserRequest {
//         organization_id: Default::default(),
//         name: test_name,
//         email: test_email.parse().unwrap(),
//         password: test_password,
//     };
//
//     let response: CreateUserResponse = client
//         .create_user(request)
//         .timeout(std::time::Duration::from_secs(5))
//         .await??;
//
//     assert_eq!(response, {});
//
//     Ok(())
// }


// -----------------------------------------------
//
// #![cfg(feature = "integration")]
//
// use grapl_utils::future_ext::GraplFutureExt;
// use organization_management::client::OrganizationManagementServiceClient;
// use organization_management::OrganizationManagementServiceConfig;
//
// use rust_proto::organization_management::{
//     CreateUserRequest,
//     // CreateOrganizationResponse,
// };
//
// use structopt::StructOpt;
//
// #[test_log::test(tokio::test)]
// async fn test_create_organization() -> Result<(), Box<dyn std::error::Error>> {
//     tracing::debug!(
//         env=?std::env::args(),
//     );
//
//     let service_config = OrganizationManagementServiceConfig::from_args();
//
//     let postgres_address = format!(
//         "postgresql://{}:{}@{}:{}",
//         service_config.organization_management_db_username,
//         service_config.organization_management_db_password,
//         service_config.organization_management_db_hostname,
//         service_config.organization_management_db_port,
//     );
//
//     let pool = sqlx::PgPool::connect(&postgres_address)
//         .timeout(std::time::Duration::from_secs(5))
//         .await??;
//
//     let mut client = OrganizationManagementServiceClient::from_env().await?;
//
//     let name = "test user".to_string();
//     let email = "testemail@email.com";
//     let password = b"t3stp@s$w0rd".to_vec();
//
//     let request = CreateUserRequest {
//         organization_id: Default::default(),
//         name,
//         email: email.parse().unwrap(),
//         password,
//     };
//
//     let response = client
//         .create_user(request)
//         .timeout(std::time::Duration::from_secs(5))
//         .await??;
//
//     let user_id = response.user_id;
//
//     let (name, email, password): (String, String, String) = sqlx::query_as(
//         r#"SELECT name, email, password
//         FROM users
//         WHERE user_id = $1
//         "#,
//     )
//         .bind(user_id)
//         .fetch_one(&pool).await?;
//
//     assert_eq!(name, "test user");
//     assert_eq!(email, "testemail@email.com");
//
//     Ok(())
// }




