use grapl_config::env_helpers::FromEnv;
use rand::Rng;
use rusoto_dynamodb::DynamoDbClient;
use url::Url;

use crate::services::{
    graphql::GraphQlEndpointUrl,
    model_plugin_deployer::ModelPluginDeployerEndpoint,
    plugin_registry::PluginRegistryEndpointUrl,
};

const KEY_SIZE: usize = 32;
pub(crate) const SESSION_TOKEN: &'static str = "SESSION_TOKEN";
pub(crate) const SESSION_TOKEN_LENGTH: usize = 32;
pub(crate) const SESSION_EXPIRATION_TIMEOUT_DAYS: i64 = 1;

fn get_env_var(name: &'static str) -> Result<String, ConfigError> {
    std::env::var(name).map_err(|source| ConfigError::MissingEnvironmentVariable {
        variable_name: name,
        source,
    })
}

fn parse_url(url: String) -> Result<Url, ConfigError> {
    Url::parse(url.as_str()).map_err(|source| ConfigError::UrlParse { url, source })
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
    pub plugin_registry_endpoint: PluginRegistryEndpointUrl,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ConfigError {
    #[error("unable to get required environment variable `{variable_name}`: {source}")]
    MissingEnvironmentVariable {
        variable_name: &'static str,
        source: std::env::VarError,
    },
    #[error("unable to parse URL `{url}`: {source}")]
    UrlParse {
        url: String,
        source: url::ParseError,
    },
}

impl Config {
    #[tracing::instrument(err)]
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_address = get_env_var("GRAPL_WEB_UI_BIND_ADDRESS")?;

        let user_auth_table_name = get_env_var("GRAPL_USER_AUTH_TABLE")?;
        let user_session_table_name = get_env_var("GRAPL_USER_SESSION_TABLE")?;

        // generate a random key for encrypting user state.
        let mut rng = rand::thread_rng();
        let session_key = rng.gen::<[u8; KEY_SIZE]>();

        let dynamodb_client = DynamoDbClient::from_env();

        let graphql_endpoint = get_env_var("GRAPL_GRAPHQL_ENDPOINT")
            .map(parse_url)?
            .map(GraphQlEndpointUrl::from)?;

        // Model Plugin Deployer endpoint backend
        let model_plugin_deployer_endpoint = get_env_var("GRAPL_MODEL_PLUGIN_DEPLOYER_ENDPOINT")
            .map(parse_url)?
            .map(ModelPluginDeployerEndpoint::from)?;

        // Plugin Registry endpoint backend URL
        let plugin_registry_endpoint = get_env_var("GRAPL_PLUGIN_REGISTRY_ENDPOINT")
            .map(parse_url)?
            .map(PluginRegistryEndpointUrl::from)?;

        Ok(Config {
            dynamodb_client,
            bind_address,
            session_key,
            user_auth_table_name,
            user_session_table_name,
            graphql_endpoint,
            model_plugin_deployer_endpoint,
            plugin_registry_endpoint,
        })
    }
}
