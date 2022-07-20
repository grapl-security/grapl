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

impl ConsumerConfig {
    /// In some test cases, where we need to consume from >1 topic,
    /// we'll manually specify the topic name in code instead of using
    /// the default KAFKA_CONSUMER_TOPIC.
    /// (Example: rust-integration-tests consumes from many topics)
    pub fn with_topic(topic: &str) -> Self {
        let app_args = std::env::args();
        let extra_args = ["--topic".to_string(), topic.to_string()];
        Self::parse_from(app_args.chain(extra_args.into_iter()))
    }
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
    /// In some applications, where we need to produce to >1 topic,
    /// we'll specify the env-var containing the topic name instead of depending
    /// on the default KAFKA_PRODUCER_TOPIC.
    /// (Example: plugin-work-queue produces to both generator and analyzer)
    pub fn with_topic_env_var(key: &'static str) -> Self {
        let topic = std::env::var(key).expect(key);
        Self::with_topic(&topic)
    }

    fn with_topic(topic: &str) -> Self {
        let app_args = std::env::args();
        let extra_args = ["--topic".to_string(), topic.to_string()];
        Self::parse_from(app_args.chain(extra_args.into_iter()))
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
}
