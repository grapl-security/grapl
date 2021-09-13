// use org_management::orgmanagement_client::OrgClient;
// use org_management::OrgRequest;
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let mut client = OrgClient::connect("http://[::1]:50051").await?;
//
//     let request = tonic::Request::new(OrgRequest {
//         name: "Tonic".into(),
//     });
//
//     let response = client.create_org(request).await?;
//
//     println!("RESPONSE={:?}", response);
//
//     Ok(())
// }
