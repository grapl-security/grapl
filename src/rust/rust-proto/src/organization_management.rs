pub use crate::graplinc::grapl::api::organization_management::v1beta1::{
    organization_management_service_client,
    organization_management_service_server,
    CreateOrganizationRequest as CreateOrganizationRequestProto,
    CreateOrganizationResponse as CreateOrganizationResponseProto,
    CreateUserRequest as CreateUserRequestProto,
    CreateUserResponse as CreateUserResponseProto,
};


#[derive(Debug, thiserror::Error)]
pub enum OrganizationManagementDeserializationError {
    #[error("Missing a required field: {0}")]
    MissingRequiredField(&'static str),
    #[error("Empty field: {0}")]
    EmptyField(&'static str),
}

fn validate_string(
    s: &str,
    field_name: &'static str,
) -> Result<(), OrganizationManagementDeserializationError> {
    if s.is_empty() {
        Err(OrganizationManagementDeserializationError::EmptyField(
            field_name,
        ))
    } else {
        Ok(())
    }
}

fn validate_bytes(
    s: &[u8],
    field_name: &'static str,
) -> Result<(), OrganizationManagementDeserializationError> {
    if s.is_empty() {
        Err(OrganizationManagementDeserializationError::EmptyField(
            field_name,
        ))
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CreateOrganizationRequest {
    pub organization_display_name: String,
    pub admin_username: String,
    pub admin_email: String,
    pub admin_password: Vec<u8>,
    pub should_reset_password: bool,
}

impl TryFrom<CreateOrganizationRequestProto> for CreateOrganizationRequest {
    type Error = OrganizationManagementDeserializationError;

    fn try_from(value: CreateOrganizationRequestProto) -> Result<Self, Self::Error> {
        validate_string(
            &value.organization_display_name,
            "CreateOrganizationRequestProto.organization_display_name",
        )?;
        validate_string(
            &value.admin_username,
            "CreateOrganizationRequestProto.username",
        )?;
        validate_string(
            &value.admin_email,
            "CreateOrganizationRequestProto.email",
        )?;
        validate_bytes(
            &value.admin_password,
            "CreateOrganizationRequestProto.password",
        )?;

        Ok(Self {
            organization_display_name: value.organization_display_name,
            admin_username: value.admin_username,
            admin_email: value.admin_email,
            admin_password: value.admin_password,
            should_reset_password: value.should_reset_password,
        })
    }
}

impl From<CreateOrganizationRequest> for CreateOrganizationRequestProto {
    fn from(value: CreateOrganizationRequest) -> Self {
        Self {
            organization_display_name: value.organization_display_name,
            admin_username: value.admin_username,
            admin_email: value.admin_email,
            admin_password: value.admin_password,
            should_reset_password: value.should_reset_password,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CreateOrganizationResponse {
    pub organization_id: uuid::Uuid,
}

impl TryFrom<CreateOrganizationResponseProto> for CreateOrganizationResponse {
    type Error = OrganizationManagementDeserializationError;

    fn try_from(value: CreateOrganizationResponseProto) -> Result<Self, Self::Error> {
        let organization_id = value
            .organization_id
            .ok_or(Self::Error::MissingRequiredField(
                "CreateUserRequest.organization_id",
            ))?
            .into();
        
        Ok(Self { organization_id })
    }
}

impl From<CreateOrganizationResponse> for CreateOrganizationResponseProto {
    fn from(value: CreateOrganizationResponse) -> Self {
        Self {
            organization_id: Some(value.organization_id.into())
        }
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
    type Error = OrganizationManagementDeserializationError;

    fn try_from(value: CreateUserRequestProto) -> Result<Self, Self::Error> {
        let organization_id = value
            .organization_id
            .ok_or(Self::Error::MissingRequiredField(
                "CreateUserRequest.organization_id",
            ))?
            .into();
        validate_string(&value.name, "CreateUserRequest.name")?;
        validate_string(&value.email, "CreateUserRequest.email")?;
        validate_bytes(&value.password, "CreateUserRequest.password")?;

        Ok(Self {
            organization_id,
            name: value.name,
            email: value.email,
            password: value.password,
        })
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
    type Error = OrganizationManagementDeserializationError;

    fn try_from(_value: CreateUserResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<CreateUserResponse> for CreateUserResponseProto {
    fn from(_value: CreateUserResponse) -> Self {
        Self {}
    }
}



