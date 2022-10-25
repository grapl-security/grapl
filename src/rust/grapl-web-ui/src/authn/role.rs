#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GraplRole {
    Owner,
    Administrator,
    User,
}

impl std::fmt::Debug for GraplRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for GraplRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            GraplRole::Owner => "owner",
            GraplRole::Administrator => "administrator",
            GraplRole::User => "user",
        };
        write!(f, "{}", value)
    }
}
