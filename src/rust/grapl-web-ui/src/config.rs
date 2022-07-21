use grapl_config::env_helpers::FromEnv;
use rand::Rng;
use rusoto_dynamodb::DynamoDbClient;

use crate::GraphQlEndpointUrl;

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

pub struct Config {
    pub dynamodb_client: DynamoDbClient,
    pub listener: std::net::TcpListener,
    pub session_key: [u8; KEY_SIZE],
    pub user_auth_table_name: String,
    pub user_session_table_name: String,
    pub graphql_endpoint: GraphQlEndpointUrl,
    pub google_client_id: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
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
    #[error(transparent)]
    BindAddress(#[from] std::io::Error),
}

impl Config {
    #[tracing::instrument(err)]
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_address =
            std::env::var("GRAPL_WEB_UI_BIND_ADDRESS").unwrap_or("127.0.0.1:1234".to_string());
        let listener = std::net::TcpListener::bind(bind_address)?;

        let user_auth_table_name = get_env_var("GRAPL_USER_AUTH_TABLE")?;
        let user_session_table_name = get_env_var("GRAPL_USER_SESSION_TABLE")?;

        // generate a random key for encrypting user state.
        let mut rng = rand::thread_rng();
        let session_key = rng.gen::<[u8; KEY_SIZE]>();

        let dynamodb_client = DynamoDbClient::from_env();

        let graphql_endpoint = get_env_var("GRAPL_GRAPHQL_ENDPOINT")
            .map(|url| {
                url::Url::parse(url.as_str())
                    .map_err(|source| ConfigError::UrlParse { url, source })
            })?
            .map(GraphQlEndpointUrl::from)?;

        let google_client_id = get_env_var("GRAPL_GOOGLE_CLIENT_ID")?;

        Ok(Config {
            dynamodb_client,
            listener,
            session_key,
            user_auth_table_name,
            user_session_table_name,
            graphql_endpoint,
            google_client_id,
        })
    }
}
