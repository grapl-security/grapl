use org_management::orgmanagement_client::OrgClient;
use org_management::OrgRequest;

pub mod org_management{
    tonic::include_proto!("orgmanagement");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrgClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(OrgRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
