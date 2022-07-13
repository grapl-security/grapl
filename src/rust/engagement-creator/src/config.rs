#[derive(clap::Parser, Debug)]
#[clap(name = "node-identifier", about = "Node Identifier Service")]
pub struct EngagementCreatorConfig {
    #[clap(env="GRAPH_MUTATION_CLIENT_URL")]
    pub graph_mutation_client_url: String,
}
