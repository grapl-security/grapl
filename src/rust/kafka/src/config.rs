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
