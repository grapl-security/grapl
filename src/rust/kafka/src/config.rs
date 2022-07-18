use clap::Parser;

#[derive(clap::Parser, Clone, Debug)]
pub struct ConsumerConfig {
    #[clap(long, env = "KAFKA_BOOTSTRAP_SERVERS")]
    pub bootstrap_servers: String,
    #[clap(long, env = "KAFKA_SASL_USERNAME")]
    pub sasl_username: String,
    #[clap(long, env = "KAFKA_SASL_PASSWORD")]
    pub sasl_password: String,
    #[clap(long, env = "KAFKA_CONSUMER_GROUP_NAME")]
    pub consumer_group_name: String,
    #[clap(long, env = "KAFKA_CONSUMER_TOPIC")]
    pub topic: String,
}

#[derive(clap::Parser, Clone, Debug)]
pub struct RetryConsumerConfig {
    #[clap(long, env = "KAFKA_BOOTSTRAP_SERVERS")]
    pub bootstrap_servers: String,
    #[clap(long, env = "KAFKA_SASL_USERNAME")]
    pub sasl_username: String,
    #[clap(long, env = "KAFKA_SASL_PASSWORD")]
    pub sasl_password: String,
    #[clap(long, env = "KAFKA_CONSUMER_GROUP_NAME")]
    pub consumer_group_name: String,
    #[clap(long, env = "KAFKA_RETRY_TOPIC")]
    pub topic: String,
}

#[derive(clap::Parser, Clone, Debug, Default)]
pub struct ProducerConfig {
    #[clap(long, env = "KAFKA_BOOTSTRAP_SERVERS")]
    pub bootstrap_servers: String,
    #[clap(long, env = "KAFKA_SASL_USERNAME")]
    pub sasl_username: String,
    #[clap(long, env = "KAFKA_SASL_PASSWORD")]
    pub sasl_password: String,
    #[clap(long, env = "KAFKA_PRODUCER_TOPIC")]
    pub topic: String,
}

impl ProducerConfig {
    // In some applications, where we need to produce to >1 topic,
    // we'll specify the env-var containing the topic name instead of depending
    // on the default KAFKA_PRODUCER_TOPIC.
    pub fn with_topic_env_var(key: &'static str) -> Self {
        let topic = std::env::var(key).expect(key);
        ProducerConfig::parse_from(["--topic".to_string(), topic])
    }
}

#[derive(clap::Parser, Clone, Debug)]
pub struct RetryProducerConfig {
    #[clap(long, env = "KAFKA_BOOTSTRAP_SERVERS")]
    pub bootstrap_servers: String,
    #[clap(long, env = "KAFKA_SASL_USERNAME")]
    pub sasl_username: String,
    #[clap(long, env = "KAFKA_SASL_PASSWORD")]
    pub sasl_password: String,
    #[clap(long, env = "KAFKA_RETRY_TOPIC")]
    pub topic: String,
    