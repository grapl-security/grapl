use crate::orgmanagementlib::CreateUserRequest;
use crate::{
    orgmanagementlib::organization_manager_client::OrganizationManagerClient
};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrganizationManagerClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(CreateUserRequest {
        name: "test".into(),
        email: "test".into(),
        password: "test".into(),
    });

    let response = client.create_user(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}