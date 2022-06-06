use kafka::{
    ConfigurationError,
    Consumer,
};
use rust_proto_new::SerDe;

#[derive(clap::Parser, Debug)]
pub struct GeneratorDispatcherConfig {
    #[clap(flatten)]
    pub kafka_config: KafkaConsumerConfig,
}

#[derive(clap::Parser, Debug)]
pub struct KafkaConsumerConfig {
    #[clap(long, env = "KAFKA_BOOTSTRAP_SERVERS")]
    bootstrap_servers: String,
    #[clap(long, env = "KAFKA_SASL_USERNAME")]
    sasl_username: String,
    #[clap(long, env = "KAFKA_SASL_PASSWORD")]
    sasl_password: String,
    #[clap(long, env = "KAFKA_CONSUMER_GROUP_NAME")]
    consumer_group_name: String,
}

pub fn build_consumer<T>(
    config: KafkaConsumerConfig,
    topic_name: String,
) -> Result<Consumer<T>, ConfigurationError>
where
    T: SerDe,
{
    Consumer::new(
        config.bootstrap_servers,
        config.sasl_username,
        config.sasl_password,
        config.consumer_group_name,
        topic_name,
    )
}
