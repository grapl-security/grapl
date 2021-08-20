use tonic::{transport::Server, Request, Response, Status};
use org_management::orgmanagement_server::{OrganizationManager, OrgManagementServer};
use org_management::{OrgReply, OrgRequest};

pub mod org_management {
    tonic::include_proto!("orgmanagement");
}

#[derive(Debug, Default)]
pub struct Organization {}

#[tonic::async_trait]
impl OrganizationManager for Organization {
    async fn say_hello(
        &self,
        request: Request<OrgRequest>,
    ) -> Result<Response<OrgReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = org_management::OrgReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let org = Organization::default();

    Server::builder()
        .add_service(OrgManagementServer::new(org))
        .serve(addr)
        .await?;

    Ok(())
}
