use grapl_config::env_helpers::FromEnv;
use rusoto_dynamodb::DynamoDbClient;

type Result<T> = std::result::Result<T, TestConfigError>;

pub struct TestConfig {
    pub dynamodb_client: DynamoDbClient,
    pub user_auth_table_name: String,
    pub user_session_table_name: String,
    pub endpoint_address: url::Url,
}

#[derive(thiserror::Error, Debug)]
pub enum TestConfigError {
    #[error("unable to get required environment variable `{variable_name}`: {source}")]
    EnvironmentVariable {
        variable_name: &'static str,
        source: std::env::VarError,
    },
    #[error("unable to parse URL for '{variable_name}' with '{value}': {source}")]
    UrlParse {
        variable_name: &'static str,
        value: String,
        source: url::ParseError,
    },
}

fn get_env_var(name: &'static str) -> Result<String> {
    std::env::var(name).map_err(|source| TestConfigError::EnvironmentVariable {
        variable_name: name,
        source,
    })
}

impl TestConfig {
    pub fn from_env() -> Result<Self> {
        let dynamodb_client = DynamoDbClient::from_env();
        let user_auth_table_name = get_env_var("GRAPL_USER_AUTH_TABLE")?;
        let user_session_table_name = get_env_var("GRAPL_USER_SESSION_TABLE")?;
        let endpoint_address_str = get_env_var("GRAPL_WEB_UI_ENDPOINT_ADDRESS")?;
        let endpoint_address = url::Url::parse(endpoint_address_str.as_str()).map_err(|e| {
            TestConfigError::UrlParse {
                variable_name: "GRAPL_WEB_UI_ENDPOINT_ADDRESS",
                value: endpoint_address_str,
                source: e,
            }
        })?;

        Ok(Self {
            dynamodb_client,
            user_auth_table_name,
            user_session_table_name,
            endpoint_address,
        })
    }
}
