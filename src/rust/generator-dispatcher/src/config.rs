use kafka::config::ConsumerConfig;

#[derive(clap::Parser, Debug)]
pub struct GeneratorDispatcherConfig {
    #[clap(flatten)]
    pub kafka_config: ConsumerConfig,
}
