use tonic::{transport::Server, Request, Response, Status};
use crate::org_management::organization_manager_server::{OrganizationManager, OrganizationManagerServer};
use crate::org_management;
use org_management::{CreateOrgReply, CreateOrgRequest};


#[derive(Debug, Default)]
pub struct Organization {}

#[tonic::async_trait]
impl OrganizationManager for Organization {
    async fn create_org(
        &self,
        request: Request<CreateOrgRequest>,
    ) -> Result<Response<CreateOrgReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = CreateOrgReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let org = Organization::default();

    Server::builder()
        .add_service(OrganizationManagerServer::new(org))
        .serve(addr)
        .await?;

    Ok(())
}
