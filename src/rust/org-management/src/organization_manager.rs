use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use sqlx::types::Uuid;
use crate::orgmanagementlib::organization_manager_server::OrganizationManager;
use crate::orgmanagementlib::{
    ChangePasswordRequest,
    CreateOrgRequest,
    CreateUserRequest,
    EmptyResp,
};

#[derive(Debug)]
pub struct OrganizationManagerRpc {
    pub pool: Pool<Postgres>,
}

#[derive(thiserror::Error, Debug)]
pub enum OrganizationManagerError {
    #[error("sql")]
    Sql(#[from] sqlx::Error),
}

impl From<OrganizationManagerError> for Status {
    fn from(e: OrganizationManagerError) -> Self {
        match e {
            OrganizationManagerError::Sql(e) => Status::internal(e.to_string()),
        }
    }
}

#[tonic::async_trait]
impl OrganizationManager for OrganizationManagerRpc {
    async fn create_org(
        &self,
        request: Request<CreateOrgRequest>,
    ) -> Result<Response<EmptyResp>, Status> {
        println!("Org request data: {:?}", &request);

        let org_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());

        let CreateOrgRequest {
            org_display_name,
            admin_username,
            admin_email,
            admin_password,
            should_reset_password,
        } = &request.into_inner();

        let row = sqlx::query!(
            r"
            INSERT INTO organization (
                org_id,
                org_display_name,
                admin_username,
                admin_email,
                admin_password,
                should_reset_password
            )
             VALUES ( $1, $2, $3, $4, $5, $6 )
        ",
        org_id,
        org_display_name,
        admin_username,
        admin_email,
        admin_password,
        should_reset_password
        ).execute(&self.pool)
            .await
            .map_err(OrganizationManagerError::from)?;

        if row.rows_affected() == 0 {
            return Err(Status::internal(
                "Organization was not created successfully",
            ));
        }

        Ok(Response::new(EmptyResp {}))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<EmptyResp>, Status> {
        println!("Org request data: {:?}", &request); // don't actually print this

        let user_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());
        let org_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());

        let CreateUserRequest {
            // org_id, // we need to do a lookup here
            name,
            email,
            password,
        } = &request.into_inner();

        let row = sqlx::query!(
            r"
            INSERT INTO users (
                user_id,
                org_id,
                name,
                email,
                password
            )
             VALUES ( $1, $2, $3, $4, $5 )
        ",
            user_id,
            org_id,
            name,
            email,
            password,
        ).execute(&self.pool)
            .await
            .map_err(OrganizationManagerError::from)?;

        if row.rows_affected() == 0 {
            return Err(Status::internal("User was not created successfully"));
        }

        Ok(Response::new(EmptyResp {}))
    }

    async fn change_password(
        &self,
        _request: Request<ChangePasswordRequest>,
    ) -> Result<Response<EmptyResp>, Status> {
        // println!("Changed password for user x: {:?}", request); // don't actually print this

        // check to see if old password matches what we have in db
        // if it passes, update with new password
        // let row = sqlx::query!(
        //     "UPDATE users SET password = $2 WHERE user_id = $1",
        //          &user_id,
        //         &organization_id,
        //         &old_password,
        //         &new_password
        // )
        //     .bind("new user")
        //     .execute(&self.pool)
        //     .await
        //     .map_err(OrganizationManagerError::from)?;
        //
        // if row.rows_affected() == 0 {
        //     return Err(Status::internal("Organization was not created successfully"));
        // }

        Ok(Response::new(EmptyResp {}))
    }
}