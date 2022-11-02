use clap::Parser;
use figment::{
    providers::Env,
    Figment,
};
use grapl_config::env_helpers::FromEnv;
use rand::Rng;
use rusoto_dynamodb::DynamoDbClient;
use rust_proto::graplinc::grapl::api::{
    client::{
        ClientConfiguration,
        ClientError,
        Connect,
    },
    pipeline_ingress::v1beta1::client::PipelineIngressClient,
    plugin_registry::v1beta1::PluginRegistryClient,
};

const KEY_SIZE: usize = 32;
pub(crate) const SESSION_TOKEN: &'static str = "SESSION_TOKEN";
pub(crate) const SESSION_TOKEN_LENGTH: usize = 32;
pub(crate) const SESSION_EXPIRATION_TIMEOUT_DAYS: i64 = 1;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ConfigError {
    #[error(transparent)]
    Clap(#[from] clap::Error),
    #[error(transparent)]
    BindAddress(#[from] std::io::Error),
    #[error("failed to initialize Plugin Regsitry client: {0}")]
    ClientError(#[from] ClientError),
    #[error("failed to do the configuring: {0}")]
    FigmentError(#[from] figment::Error),
}

pub struct Config {
    pub dynamodb_client: DynamoDbClient,
    pub listener: std::net::TcpListener,
    pub session_key: [u8; KEY_SIZE],
    pub user_auth_table_name: String,
    pub user_session_table_name: String,
    pub plugin_registry_client: PluginRegistryClient,
    pub pipeline_ingress_client: PipelineIngressClient,
    pub google_client_id: String,
}

impl Config {
    pub async fn from_env() -> Result<Self, ConfigError> {
        let builder = ConfigBuilder::try_parse()?;

        let listener = std::net::TcpListener::bind(builder.bind_address)?;

        let plugin_registry_config: ClientConfiguration = Figment::new()
            .merge(Env::prefixed("PLUGIN_REGISTRY_CLIENT_"))
            .extract()?;

        let plugin_registry_client = PluginRegistryClient::connect(plugin_registry_config).await?;

        let pipeline_ingress_config: ClientConfiguration = Figment::new()
            .merge(Env::prefixed("PIPELINE_INGRESS_CLIENT_"))
            .extract()?;

        let pipeline_ingress_client =
            PipelineIngressClient::connect(pipeline_ingress_config).await?;

        let dynamodb_client = DynamoDbClient::from_env();

        // generate a random key for encrypting user state.
        let session_key = rand::thread_rng().gen::<[u8; KEY_SIZE]>();

        let config = Config {
            dynamodb_client,
            listener,
            session_key,
            user_auth_table_name: builder.user_auth_table_name,
            user_session_table_name: builder.user_session_table_name,
            plugin_registry_client,
            pipeline_ingress_client,
            google_client_id: builder.google_client_id,
        };

        Ok(config)
    }
}

#[derive(clap::Parser, Debug)]
#[clap(name = "grapl-web-ui", about = "Grapl web")]
pub struct ConfigBuilder {
    #[clap(env = "GRAPL_WEB_UI_BIND_ADDRESS")]
    pub bind_address: String,
    #[clap(env = "GRAPL_USER_AUTH_TABLE")]
    pub user_auth_table_name: String,
    #[clap(env = "GRAPL_USER_SESSION_TABLE")]
    pub user_session_table_name: String,
    #[clap(env = "GRAPL_GOOGLE_CLIENT_ID")]
    pub google_client_id: String,
}
