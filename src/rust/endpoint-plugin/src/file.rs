use derive_dynamic_node::{GraplSessionId,
                          NodeDescription};
use grapl_graph_descriptions::graph_description::*;

#[derive(NodeDescription, GraplSessionId)]
pub struct File {
    #[grapl(pseudo_key, immutable)]
    asset_id: String,
    #[grapl(pseudo_key, immutable)]
    file_path: String,
    #[grapl(create_time, immutable)]
    created_timestamp: u64,
    #[grapl(terminate_time, immutable)]
    deleted_timestamp: u64,
    #[grapl(last_seen_time, increment)]
    last_seen_timestamp: u64,
    #[grapl(immutable)]
    file_name: String,
    #[grapl(immutable)]
    file_extension: String,
    #[grapl(immutable)]
    file_mime_type: String,
    #[grapl(immutable)]
    file_description: String,
    #[grapl(immutable)]
    file_product: String,
    #[grapl(immutable)]
    file_company: String,
    #[grapl(immutable)]
    file_directory: String,
    #[grapl(immutable)]
    file_inode: u64,
}

impl IFileNode for FileNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}

//
// todo: Not ready yet - should create an issue/ RFC instead
// // A point in time snapshot of file state
// #[derive(NodeDescription, GraplStaticId)]
// pub struct FileState {
//     #[grapl(static_id, immutable)]
//     snapshot_timestamp: u64,
//     #[grapl(static_id, immutable)]
//     file_size: u64,
//     #[grapl(static_id, immutable)]
//     file_hard_links: u64,
//     #[grapl(static_id, immutable)]
//     md5_hash: String,
//     #[grapl(static_id, immutable)]
//     sha1_hash: String,
//     #[grapl(static_id, immutable)]
//     sha256_hash: String,
//     #[grapl(static_id, immutable)]
//     file_version: String,
// }
