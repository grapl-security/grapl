use rust_proto::client_factory::services::GraphMutationClientConfig;

#[derive(clap::Parser, Debug)]
pub struct GraphMergerConfig {
    #[clap(flatten)]
    pub graph_mutation_client_config: GraphMutationClientConfig,
}
