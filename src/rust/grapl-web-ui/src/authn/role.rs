use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GraplRole {
    Owner,
    Administrator,
    User,
}

impl ToString for GraplRole {
    fn to_string(&self) -> String {
        match self {
            GraplRole::Owner => "owner".to_string(),
            GraplRole::Administrator => "administrator".to_string(),
            GraplRole::User => "user".to_string(),
        }
    }
}

impl std::fmt::Debug for GraplRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
