use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;


use crate::org_management::organization_manager_server::{OrganizationManager, OrganizationManagerServer};
use crate::org_management;
use org_management::{CreateOrgReply, CreateOrgRequest};

#[derive(Debug, Default)]
pub struct Organization {}
pub struct User {}

#[tonic::async_trait]
impl OrganizationManager for Organization {
    async fn create_org(
        &self,
        request: Request<CreateOrgRequest>,
    ) -> Result<Response<CreateOrgReply>, Status> {
        println!("Org request data: {:?}", request); // don't actually print this

        let org_id =  Uuid::new_v4();

        // store data in dynamo db with new org id

        let reply = CreateOrgReply {
            organization_id: format!("Org Id {} Created", org_id).into(),
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
