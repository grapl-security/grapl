use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(Debug, Clone, PartialEq, Hash, NodeDescription, GraplStaticId)]
struct Asset {
    #[grapl(static_id, immutable)]
    asset_id: String,
    #[grapl(immutable)]
    hostname: String,
    #[grapl(immutable)]
    os_type: String,
}

impl IAssetNode for AssetNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}

impl From<&sysmon_parser::System<'_>> for AssetNode {
    fn from(system: &sysmon_parser::System) -> Self {
        let mut asset = AssetNode::new(AssetNode::static_strategy());

        asset
            .with_asset_id(&system.computer)
            .with_hostname(&system.computer);

        let os_type = match system.provider.name {
            Some(std::borrow::Cow::Borrowed("Linux-Sysmon")) => Some("linux"),
            Some(std::borrow::Cow::Borrowed("Microsoft-Windows-Sysmon")) => Some("windows"),
            _ => None, // unexpected
        };
    
        if let Some(os_type) = os_type {
            asset.with_os_type(os_type.to_string());
        }

        asset
    }
}