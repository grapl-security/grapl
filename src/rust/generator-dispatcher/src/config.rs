use kafka::config::KafkaConsumerConfig;

#[derive(clap::Parser, Debug)]
pub struct GeneratorDispatcherConfig {
    #[clap(flatten)]
    pub kafka_config: KafkaConsumerConfig,
}
