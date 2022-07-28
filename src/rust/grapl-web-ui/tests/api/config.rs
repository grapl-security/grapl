use clap::Parser;
use grapl_config::env_helpers::FromEnv;
use rusoto_dynamodb::DynamoDbClient;

pub struct TestConfig {
    pub dynamodb_client: DynamoDbClient,
    pub user_auth_table_name: String,
    pub user_session_table_name: String,
    pub endpoint_address: url::Url,
}

impl TestConfig {
    pub fn from_env() -> Result<Self, clap::Error> {
        let builder = TestConfigBuilder::try_parse()?;

        let dynamodb_client = DynamoDbClient::from_env();

        Ok(Self {
            dynamodb_client,
            user_auth_table_name: builder.user_auth_table_name,
            user_session_table_name: builder.user_session_table_name,
            endpoint_address: builder.endpoint_address,
        })
    }
}

#[derive(clap::Parser)]
#[clap(name = "grapl-web-ui tests", about = "Grapl web integration tests")]
pub struct TestConfigBuilder {
    #[clap(env = "GRAPL_USER_AUTH_TABLE")]
    pub user_auth_table_name: String,
    #[clap(env = "GRAPL_USER_SESSION_TABLE")]
    pub user_session_table_name: String,
    #[clap(env = "GRAPL_WEB_UI_ENDPOINT_ADDRESS")]
    pub endpoint_address: url::Url,
}
