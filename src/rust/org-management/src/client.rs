use crate::orgmanagementlib::CreateUserRequest;
use crate::orgmanagement::OrgManagementClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrgManagementClient::connect("http://[::1]:50051")?;

    let request = tonic::Request::new(CreateUserRequest {
        name: "test".into(),
        email: "test".into(),
        password: "test".into(),
    });

    let response = client.create_user(request).await?;

    println!("RESPONSE={:?}", response);
    // let user_date_of_birth = &response.into_inner().date_of_birth;
    // println!("{}", user_date_of_birth);

    Ok(())
}