#[derive(clap::Parser, Debug)]
#[clap(name = "graph-merger", about = "Graph Merger Service")]
pub struct GraphMergerConfig {
    #[clap(env = "GRAPH_MUTATION_CLIENT_URL")]
    pub graph_mutation_client_url: String,
}
