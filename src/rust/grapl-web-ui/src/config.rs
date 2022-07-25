use grapl_config::env_helpers::FromEnv;
use rand::Rng;
use rusoto_dynamodb::DynamoDbClient;

use crate::GraphQlEndpointUrl;

const KEY_SIZE: usize = 32;
pub(crate) const SESSION_TOKEN: &'static str = "SESSION_TOKEN";
pub(crate) const SESSION_TOKEN_LENGTH: usize = 32;
pub(crate) const SESSION_EXPIRATION_TIMEOUT_DAYS: i64 = 1;

fn get_env_var(name: &'static str) -> Result<String, ConfigError> {
    std::env::var(name).map_err(|source| ConfigError::EnvironmentVariable {
        variable_name: name,
        source,
    })
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
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
    #[error(transparent)]
    BindAddress(#[from] std::io::Error),
    #[error("ConfigBuilder missing TcpListener")]
    MissingTcpListener,
}

pub struct Config {
    pub dynamodb_client: DynamoDbClient,
    pub listener: std::net::TcpListener,
    pub session_key: [u8; KEY_SIZE],
    pub user_auth_table_name: String,
    pub user_session_table_name: String,
    pub graphql_endpoint: GraphQlEndpointUrl,
    pub google_client_id: String,
}

pub struct ConfigBuilder {
    pub dynamodb_client: DynamoDbClient,
    pub listener: Option<std::net::TcpListener>,
    pub session_key: [u8; KEY_SIZE],
    pub user_auth_table_name: String,
    pub user_session_table_name: String,
    pub graphql_endpoint: GraphQlEndpointUrl,
    pub google_client_id: String,
}

impl ConfigBuilder {
    #[tracing::instrument(err)]
    pub fn from_env() -> Result<Self, ConfigError> {
        let listener = std::env::var("GRAPL_WEB_UI_BIND_ADDRESS")
            .ok()
            .map(std::net::TcpListener::bind)
            .transpose()?;

        let user_auth_table_name = get_env_var("GRAPL_USER_AUTH_TABLE")?;
        let user_session_table_name = get_env_var("GRAPL_USER_SESSION_TABLE")?;

        // generate a random key for encrypting user state.
        let session_key = rand::thread_rng().gen::<[u8; KEY_SIZE]>();

        let dynamodb_client = DynamoDbClient::from_env();

        let graphql_endpoint = get_env_var("GRAPL_GRAPHQL_ENDPOINT")
            .map(|url| {
                url::Url::parse(url.as_str()).map_err(|source| ConfigError::UrlParse {
                    variable_name: "GRAPL_GRAPHQL_ENDPOINT",
                    value: url,
                    source,
                })
            })?
            .map(GraphQlEndpointUrl::from)?;

        let google_client_id = get_env_var("GRAPL_GOOGLE_CLIENT_ID")?;

        Ok(ConfigBuilder {
            dynamodb_client,
            listener,
            session_key,
            user_auth_table_name,
            user_session_table_name,
            graphql_endpoint,
            google_client_id,
        })
    }

    pub fn with_listener(mut self, listener: std::net::TcpListener) -> Self {
        self.listener = Some(listener);

        self
    }

    pub fn build(self) -> Result<Config, ConfigError> {
        let config = Config {
            dynamodb_client: self.dynamodb_client,
            listener: self
                .listener
                .ok_or_else(|| ConfigError::MissingTcpListener)?,
            session_key: self.session_key,
            user_auth_table_name: self.user_auth_table_name,
            user_session_table_name: self.user_session_table_name,
            graphql_endpoint: self.graphql_endpoint,
            google_client_id: self.google_client_id,
        };

        Ok(config)
    }
}
