use rust_proto::client_factory::services::GraphMutationClientConfig;

#[derive(clap::Parser, Debug)]
#[clap(name = "node-identifier", about = "Node Identifier Service")]
pub struct NodeIdentifierConfig {
    #[clap(long, env)]
    pub grapl_static_mapping_table: String,
    #[clap(long, env)]
    pub grapl_dynamic_session_table: String,
    #[clap(flatten)]
    pub graph_mutation_client_config: GraphMutationClientConfig,
}
