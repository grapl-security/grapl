#[derive(clap::Parser, Debug)]
#[clap(name = "node-identifier", about = "Node Identifier Service")]
pub struct NodeIdentifierConfig {
    #[clap(env)]
    pub grapl_static_mapping_table: String,
    #[clap(env)]
    pub graph_mutation_url: String,
    #[clap(env)]
    pub grapl_dynamic_session_table: String,
}
