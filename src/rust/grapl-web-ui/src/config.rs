use grapl_config::env_helpers::FromEnv;
use rand::Rng;
use rusoto_dynamodb::DynamoDbClient;
use url::Url;

use crate::services::{
    graphql::GraphQlEndpointUrl,
    model_plugin_deployer::ModelPluginDeployerEndpoint,
};

const KEY_SIZE: usize = 32;
pub(crate) const SESSION_TOKEN: &'static str = "SESSION_TOKEN";
pub(crate) const SESSION_TOKEN_LENGTH: usize = 32;
pub(crate) const SESSION_EXPIRATION_TIMEOUT_DAYS: i64 = 1;

// Try getting an environment variable and create ConfigError::MissingEnvironmentVariable if
// unsuccessful. The benefit of this over just std::env::VarError is the environment variable will
// be Display'd as well.
macro_rules! env_var {
    ($name:expr) => {
        std::env::var($name)
            .map_err(|var_err| ConfigError::MissingEnvironmentVariable($name, var_err))
    };
}

#[derive(Clone)]
pub(crate) struct Config {
    pub dynamodb_client: DynamoDbClient,
    pub bind_address: String,
    pub session_key: [u8; KEY_SIZE],
    pub user_auth_table_name: String,
    pub user_session_table_name: String,
    pub graphql_endpoint: GraphQlEndpointUrl,
    pub model_plugin_deployer_endpoint: ModelPluginDeployerEndpoint,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ConfigError {
    #[error("required environment variable '{0}' error: {1}")]
    MissingEnvironmentVariable(&'static str, std::env::VarError),
    #[error("unable to parse URL: `{0}`")]
    UrlParse(#[from] url::ParseError),
    #[error("unable to parse AWS region: `{0}`")]
    AwsRegionParse(#[from] rusoto_core::region::ParseRegionError),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_address = env_var!("GRAPL_WEB_UI_BIND_ADDRESS")?;

        let user_auth_table_name = env_var!("GRAPL_USER_AUTH_TABLE")?;
        let user_session_table_name = env_var!("GRAPL_USER_SESSION_TABLE")?;

        // generate a random key for encrypting user state.
        let mut rng = rand::thread_rng();
        let session_key = rng.gen::<[u8; KEY_SIZE]>();

        let dynamodb_client = DynamoDbClient::from_env();

        // GraphQL endpoint backend
        let graphql_endpoint = env_var!("GRAPL_GRAPHQL_ENDPOINT")?;
        let graphql_endpoint = Url::parse(graphql_endpoint.as_str())?;
        let graphql_endpoint = GraphQlEndpointUrl::from(graphql_endpoint);

        // Model Plugin Deployer endpoint backend
        let model_plugin_deployer_endpoint = env_var!("GRAPL_MODEL_PLUGIN_DEPLOYER_ENDPOINT")?;
        let model_plugin_deployer_endpoint = Url::parse(model_plugin_deployer_endpoint.as_str())?;
        let model_plugin_deployer_endpoint =
            ModelPluginDeployerEndpoint::from(model_plugin_deployer_endpoint);

        Ok(Config {
            dynamodb_client,
            bind_address,
            session_key,
            user_auth_table_name,
            user_session_table_name,
            graphql_endpoint,
            model_plugin_deployer_endpoint,
        })
    }
}
