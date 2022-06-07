#[derive(clap::Parser, Clone, Debug)]
pub struct KafkaConsumerConfig {
    #[clap(long, env = "KAFKA_BOOTSTRAP_SERVERS")]
    pub bootstrap_servers: String,
    #[clap(long, env = "KAFKA_SASL_USERNAME")]
    pub sasl_username: String,
    #[clap(long, env = "KAFKA_SASL_PASSWORD")]
    pub sasl_password: String,
    #[clap(long, env = "KAFKA_CONSUMER_GROUP_NAME")]
    pub consumer_group_name: String,
}

#[derive(clap::Parser, Clone, Debug)]
pub struct KafkaProducerConfig {
    #[clap(long, env = "KAFKA_BOOTSTRAP_SERVERS")]
    pub bootstrap_servers: String,
    #[clap(long, env = "KAFKA_SASL_USERNAME")]
    pub sasl_username: String,
    #[clap(long, env = "KAFKA_SASL_PASSWORD")]
    pub sasl_password: String,
}
