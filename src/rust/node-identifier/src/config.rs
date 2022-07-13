#[derive(clap::Parser, Debug)]
#[clap(name = "node-identifier", about = "Node Identifier Service")]
pub struct NodeIdentifierConfig {
    #[clap(env="GRAPL_STATIC_MAPPING_TABLE")]
    pub grapl_static_mapping_table: String,
    #[clap(env="UID_ALLOCATOR_URL")]
    pub uid_allocator_url: String,
    #[clap(env="GRAPL_DYNAMIC_SESSION_TABLE")]
    pub grapl_dynamic_session_table: String,
}
