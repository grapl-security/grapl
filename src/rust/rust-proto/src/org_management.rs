pub use crate::graplinc::grapl::api::org_management::v1beta1::{
    // organization_manager_service_client,
    org_management_service_server,
    CreateOrgRequest as CreateOrgRequestProto,
    CreateUserRequest as CreateUserRequestProto,
    ChangePasswordRequest as ChangePasswordRequestProto,
    CreateOrgResponse as CreateOrgResponseProto,
    CreateUserResponse as CreateUserResponseProto,
    ChangePasswordResponse as ChangePasswordResponseProto,
};


#[derive(Debug, thiserror::Error)]
pub enum OrgManagementDeserializationError {
    #[error("Missing a required field: {0}")]
    MissingRequiredField(&'static str),
    #[error("Empty field: {0}")]
    EmptyField(&'static str),
}

fn validate_string(s: &str, field_name: &'static str) -> Result<(), OrgManagementDeserializationError> {
    if s.is_empty() {
        Err(OrgManagementDeserializationError::EmptyField(field_name))
    } else {
        Ok(())
    }
}

fn validate_bytes(s: &[u8], field_name: &'static str) -> Result<(), OrgManagementDeserializationError> {
    if s.is_empty() {
        Err(OrgManagementDeserializationError::EmptyField(field_name))
    } else {
        Ok(())
    }
}


#[derive(Clone)]
pub struct CreateOrgRequest {
    pub org_display_name: String,
    pub admin_username: String,
    pub admin_email: String,
    pub admin_password: Vec<u8>,
    pub should_reset_password: bool,
}

impl TryFrom<CreateOrgRequestProto> for CreateOrgRequest {
    type Error = OrgManagementDeserializationError;

    fn try_from(value: CreateOrgRequestProto) -> Result<Self, Self::Error> {
        validate_string(&value.org_display_name, "CreateOrgRequestProto.org_display_name")?;
        validate_string(&value.admin_username, "CreateOrgRequestProto.admin_username")?;
        validate_string(&value.admin_email, "CreateOrgRequestProto.admin_email")?;
        validate_bytes(&value.admin_password, "CreateOrgRequestProto.admin_password")?;

        Ok(
            Self {
                org_display_name: value.org_display_name,
                admin_username: value.admin_username,
                admin_email: value.admin_email,
                admin_password: value.admin_password,
                should_reset_password: value.should_reset_password,
            }
        )
    }
}

impl From<CreateOrgRequest> for CreateOrgRequestProto {
    fn from(value: CreateOrgRequest) -> Self {
        Self {
            org_display_name: value.org_display_name,
            admin_username: value.admin_username,
            admin_email: value.admin_email,
            admin_password: value.admin_password,
            should_reset_password: value.should_reset_password,
        }
    }
}

#[derive(Clone)]
pub struct CreateOrgResponse {}

impl TryFrom<CreateOrgResponseProto> for CreateOrgResponse {
    type Error = OrgManagementDeserializationError;

    fn try_from(_value: CreateOrgResponseProto) -> Result<Self, Self::Error> {
        Ok(
            Self {}
        )
    }
}

impl From<CreateOrgResponse> for CreateOrgResponseProto {
    fn from(_value: CreateOrgResponse) -> Self {
        Self {}
    }
}


#[derive(Clone)]
pub struct CreateUserRequest {
    pub organization_id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub password: Vec<u8>,
}


impl TryFrom<CreateUserRequestProto> for CreateUserRequest {
    type Error = OrgManagementDeserializationError;

    fn try_from(value: CreateUserRequestProto) -> Result<Self, Self::Error> {
        let organization_id = value.organization_id
            .ok_or(Self::Error::MissingRequiredField(
                "CreateUserRequest.organization_id",
            ))?
            .into();
        validate_string(&value.name, "CreateUserRequest.name")?;
        validate_string(&value.email, "CreateUserRequest.email")?;
        validate_bytes(&value.password, "CreateUserRequest.password")?;

        Ok(
            Self {
                organization_id,
                name: value.name,
                email: value.email,
                password: value.password,
            }
        )
    }
}

impl From<CreateUserRequest> for CreateUserRequestProto {
    fn from(value: CreateUserRequest) -> Self {
        Self {
            organization_id: Some(value.organization_id.into()),
            name: value.name,
            email: value.email,
            password: value.password,
        }
    }
}

#[derive(Clone)]
pub struct CreateUserResponse {}


impl TryFrom<CreateUserResponseProto> for CreateUserResponse {
    type Error = OrgManagementDeserializationError;

    fn try_from(_value: CreateUserResponseProto) -> Result<Self, Self::Error> {
        Ok(
            Self {}
        )
    }
}

impl From<CreateUserResponse> for CreateUserResponseProto {
    fn from(_value: CreateUserResponse) -> Self {
        Self {}
    }
}


#[derive(Clone)]
pub struct ChangePasswordRequest {
    pub user_id: uuid::Uuid,
    pub old_password: Vec<u8>,
    pub new_password: Vec<u8>,
}

impl TryFrom<ChangePasswordRequestProto> for ChangePasswordRequest {
    type Error = OrgManagementDeserializationError;

    fn try_from(value: ChangePasswordRequestProto) -> Result<Self, Self::Error> {
        let user_id = value.user_id
            .ok_or(Self::Error::MissingRequiredField(
                "ChangePasswordRequest.user_id",
            ))?
            .into();
        validate_bytes(&value.old_password, "ChangePasswordRequest.old_password")?;
        validate_bytes(&value.new_password, "ChangePasswordRequest.new_password")?;

        Ok(
            Self {
                user_id,
                old_password: value.old_password,
                new_password: value.new_password,
            }
        )
    }
}

impl From<ChangePasswordRequest> for ChangePasswordRequestProto {
    fn from(value: ChangePasswordRequest) -> Self {
        Self {
            user_id: Some(value.user_id.into()),
            old_password: value.old_password,
            new_password: value.new_password,
        }
    }
}

#[derive(Clone)]
pub struct ChangePasswordResponse {}

impl TryFrom<ChangePasswordResponseProto> for ChangePasswordResponse {
    type Error = OrgManagementDeserializationError;

    fn try_from(_value: ChangePasswordResponseProto) -> Result<Self, Self::Error> {
        Ok(
            Self {}
        )
    }
}

impl From<ChangePasswordResponse> for ChangePasswordResponseProto {
    fn from(_value: ChangePasswordResponse) -> Self {
        Self {}
    }
}



