use graph_description::{Node, Asset, Process, File, IpAddress, ProcessOutboundConnection, ProcessInboundConnection, IpPort, NetworkConnection, DynamicNode, IpConnection};

use graph_description::node::WhichNode;
use serde_json::Value;

pub trait NodeT {
    fn get_asset_id(&self) -> Option<&str>;

    fn clone_asset_id(&self) -> Option<String> {
        self.get_asset_id().map(String::from)
    }

    fn set_asset_id(&mut self, asset_id: impl Into<String>);

    fn get_node_key(&self) -> &str;

    fn clone_node_key(&self) -> String {
        self.get_node_key().to_string()
    }

    fn set_node_key(&mut self, node_key: impl Into<String>);

    fn merge(&mut self, other: &Self) -> bool;

    fn merge_into(&mut self, other: Self) -> bool;
}

impl From<IpConnection> for Node {
    fn from(ip_connection: IpConnection) -> Self {
        Self {
            which_node: Some(
                WhichNode::IpConnectionNode(
                    ip_connection
                )
            )
        }
    }
}

impl From<Asset> for Node {
    fn from(asset: Asset) -> Self {
        Self {
            which_node: Some(
                WhichNode::AssetNode(
                    asset
                )
            )
        }
    }
}

impl From<Process> for Node {
    fn from(process: Process) -> Self {
        Self {
            which_node: Some(
                WhichNode::ProcessNode(
                    process
                )
            )
        }
    }
}

impl From<File> for Node {
    fn from(file: File) -> Self {
        Self {
            which_node: Some(
                WhichNode::FileNode(
                    file
                )
            )
        }
    }
}

impl From<IpAddress> for Node {
    fn from(ip_address: IpAddress) -> Self {
        Self {
            which_node: Some(
                WhichNode::IpAddressNode(
                    ip_address
                )
            )
        }
    }
}

impl From<ProcessOutboundConnection> for Node {
    fn from(process_outbound_connection: ProcessOutboundConnection) -> Self {
        Self {
            which_node: Some(
                WhichNode::ProcessOutboundConnectionNode(
                    process_outbound_connection
                )
            )
        }
    }
}

impl From<ProcessInboundConnection> for Node {
    fn from(process_inbound_connection: ProcessInboundConnection) -> Self {
        Self {
            which_node: Some(
                WhichNode::ProcessInboundConnectionNode(
                    process_inbound_connection
                )
            )
        }
    }
}

impl From<IpPort> for Node {
    fn from(ip_port: IpPort) -> Self {
        Self {
            which_node: Some(
                WhichNode::IpPortNode(
                    ip_port
                )
            )
        }
    }
}

impl From<NetworkConnection> for Node {
    fn from(network_connection: NetworkConnection) -> Self {
        Self {
            which_node: Some(
                WhichNode::NetworkConnectionNode(
                    network_connection
                )
            )
        }
    }
}

impl From<DynamicNode> for Node {
    fn from(dynamic_node: DynamicNode) -> Self {
        Self {
            which_node: Some(
                WhichNode::DynamicNode(
                    dynamic_node
                )
            )
        }
    }
}


impl Node {
    pub fn as_asset(&self) -> Option<&Asset> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::AssetNode(ref asset) = which_node {
            Some(asset)
        } else {
            None
        }
    }

    pub fn into_asset(self) -> Option<Asset> {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => return None
        };

        if let WhichNode::AssetNode(asset) = which_node {
            Some(asset)
        } else {
            None
        }
    }

    pub fn as_mut_asset(&mut self) -> Option<&mut Asset> {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => return None
        };

        if let WhichNode::AssetNode(ref mut asset) = which_node {
            Some(asset)
        } else {
            None
        }
    }

    pub fn as_process(&self) -> Option<&Process> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::ProcessNode(ref process) = which_node {
            Some(process)
        } else {
            None
        }
    }

    pub fn into_process(self) -> Option<Process> {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => return None
        };

        if let WhichNode::ProcessNode(process) = which_node {
            Some(process)
        } else {
            None
        }
    }

    pub fn as_mut_process(&mut self) -> Option<&mut Process> {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => return None
        };

        if let WhichNode::ProcessNode(ref mut process) = which_node {
            Some(process)
        } else {
            None
        }
    }

    pub fn as_file(&self) -> Option<&File> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::FileNode(ref file) = which_node {
            Some(file)
        } else {
            None
        }
    }

    pub fn into_file(self) -> Option<File> {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => return None
        };

        if let WhichNode::FileNode(file) = which_node {
            Some(file)
        } else {
            None
        }
    }

    pub fn as_mut_file(&mut self) -> Option<&mut File> {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => return None
        };

        if let WhichNode::FileNode(ref mut file) = which_node {
            Some(file)
        } else {
            None
        }
    }

    pub fn as_ip_address(&self) -> Option<&IpAddress> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::IpAddressNode(ref ip_address) = which_node {
            Some(ip_address)
        } else {
            None
        }
    }

    pub fn into_ip_address(self) -> Option<IpAddress> {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => return None
        };

        if let WhichNode::IpAddressNode(ip_address) = which_node {
            Some(ip_address)
        } else {
            None
        }
    }

    pub fn as_mut_ip_address(&mut self) -> Option<&mut IpAddress> {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => return None
        };

        if let WhichNode::IpAddressNode(ref mut ip_address) = which_node {
            Some(ip_address)
        } else {
            None
        }
    }

    pub fn as_process_outbound_connection(&self) -> Option<&ProcessOutboundConnection> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::ProcessOutboundConnectionNode(ref process_outbound_connection) = which_node {
            Some(process_outbound_connection)
        } else {
            None
        }
    }

    pub fn into_process_outbound_connection(self) -> Option<ProcessOutboundConnection> {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => return None
        };

        if let WhichNode::ProcessOutboundConnectionNode(process_outbound_connection) = which_node {
            Some(process_outbound_connection)
        } else {
            None
        }
    }

    pub fn as_mut_process_outbound_connection(&mut self) -> Option<&mut ProcessOutboundConnection> {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => return None
        };

        if let WhichNode::ProcessOutboundConnectionNode(ref mut process_outbound_connection) = which_node {
            Some(process_outbound_connection)
        } else {
            None
        }
    }

    pub fn as_process_inbound_connection(&self) -> Option<&ProcessInboundConnection> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::ProcessInboundConnectionNode(ref process_inbound_connection) = which_node {
            Some(process_inbound_connection)
        } else {
            None
        }
    }

    pub fn into_process_inbound_connection(self) -> Option<ProcessInboundConnection> {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => return None
        };

        if let WhichNode::ProcessInboundConnectionNode(process_inbound_connection) = which_node {
            Some(process_inbound_connection)
        } else {
            None
        }
    }

    pub fn as_mut_process_inbound_connection(&mut self) -> Option<&mut ProcessInboundConnection> {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => return None
        };

        if let WhichNode::ProcessInboundConnectionNode(ref mut process_inbound_connection) = which_node {
            Some(process_inbound_connection)
        } else {
            None
        }
    }

    pub fn as_ip_port(&self) -> Option<&IpPort> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::IpPortNode(ref ip_port) = which_node {
            Some(ip_port)
        } else {
            None
        }
    }

    pub fn as_network_connection(&self) -> Option<&NetworkConnection> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::NetworkConnectionNode(ref network_connection) = which_node {
            Some(network_connection)
        } else {
            None
        }
    }

    pub fn into_network_connection(self) -> Option<NetworkConnection> {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => return None
        };

        if let WhichNode::NetworkConnectionNode(network_connection) = which_node {
            Some(network_connection)
        } else {
            None
        }
    }

    pub fn as_mut_network_connection(&mut self) -> Option<&mut NetworkConnection> {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => return None
        };

        if let WhichNode::NetworkConnectionNode(ref mut network_connection) = which_node {
            Some(network_connection)
        } else {
            None
        }
    }


    pub fn as_ip_connection(&self) -> Option<&IpConnection> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::IpConnectionNode(ref ip_connection) = which_node {
            Some(ip_connection)
        } else {
            None
        }
    }

    pub fn into_ip_connection(self) -> Option<IpConnection> {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => return None
        };

        if let WhichNode::IpConnectionNode(ip_connection) = which_node {
            Some(ip_connection)
        } else {
            None
        }
    }

    pub fn as_mut_ip_connection(&mut self) -> Option<&mut IpConnection> {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => return None
        };

        if let WhichNode::IpConnectionNode(ref mut ip_connection) = which_node {
            Some(ip_connection)
        } else {
            None
        }
    }

    pub fn as_dynamic_node(&self) -> Option<&DynamicNode> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => return None
        };

        if let WhichNode::DynamicNode(ref dynamic_node) = which_node {
            Some(dynamic_node)
        } else {
            None
        }
    }

    pub fn into_dynamic_node(self) -> Option<DynamicNode> {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => return None
        };

        if let WhichNode::DynamicNode(dynamic_node) = which_node {
            Some(dynamic_node)
        } else {
            None
        }
    }

    pub fn as_mut_dynamic_node(&mut self) -> Option<&mut DynamicNode> {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => {
                warn!("Failed to determine variant of node");
                return None;
            }
        };

        if let WhichNode::DynamicNode(ref mut dynamic_node) = which_node {
            Some(dynamic_node)
        } else {
            None
        }
    }

    pub fn into_json(self) -> Value {
        let which_node = match self.which_node {
            Some(which_node) => which_node,
            None => {
                panic!("Failed to determine variant of node");
            }
        };

        match which_node {
            WhichNode::AssetNode(asset_node) => {
                asset_node.into_json()
            },
            WhichNode::ProcessNode(process_node) => {
                process_node.into_json()
            },
            WhichNode::FileNode(file_node) => {
                file_node.into_json()
            },
            WhichNode::IpAddressNode(ip_address_node) => {
                ip_address_node.into_json()
            },
            WhichNode::ProcessOutboundConnectionNode(process_outbound_connection_node) => {
                process_outbound_connection_node.into_json()
            },
            WhichNode::ProcessInboundConnectionNode(process_inbound_connection_node) => {
                process_inbound_connection_node.into_json()
            },
            WhichNode::IpPortNode(ip_port_node) => {
                ip_port_node.into_json()
            },
            WhichNode::NetworkConnectionNode(network_connection_node) => {
                network_connection_node.into_json()
            },
            WhichNode::IpConnectionNode(ip_connection_node) => {
                ip_connection_node.into_json()
            },
            WhichNode::DynamicNode(dynamic_node) => {
                dynamic_node.into_json()
            },
        }

    }

}

impl NodeT for Node {
    fn get_asset_id(&self) -> Option<&str> {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => {
                warn!("Failed to determine variant of node");
                return None;
            }
        };

        match which_node {
            WhichNode::AssetNode(asset_node) => {
                asset_node.get_asset_id()
            },
            WhichNode::ProcessNode(process_node) => {
                process_node.get_asset_id()
            },
            WhichNode::FileNode(file_node) => {
                file_node.get_asset_id()
            },
            WhichNode::IpAddressNode(ip_address_node) => {
                ip_address_node.get_asset_id()
            },
            WhichNode::ProcessOutboundConnectionNode(process_outbound_connection_node) => {
                process_outbound_connection_node.get_asset_id()
            },
            WhichNode::ProcessInboundConnectionNode(process_inbound_connection_node) => {
                process_inbound_connection_node.get_asset_id()
            },
            WhichNode::IpPortNode(ip_port_node) => {
                ip_port_node.get_asset_id()
            },
            WhichNode::NetworkConnectionNode(network_connection_node) => {
                network_connection_node.get_asset_id()
            },
            WhichNode::IpConnectionNode(ip_connection_node) => {
                ip_connection_node.get_asset_id()
            },
            WhichNode::DynamicNode(dynamic_node) => {
                dynamic_node.get_asset_id()
            },
        }
    }

    fn set_asset_id(&mut self, asset_id: impl Into<String>) {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => {
                warn!("Failed to determine variant of node");
                return;
            }
        };

        match which_node {
            WhichNode::AssetNode(ref mut asset_node) => {
                asset_node.set_asset_id(asset_id.into())
            },
            WhichNode::ProcessNode(ref mut process_node) => {
                process_node.set_asset_id(asset_id.into())
            },
            WhichNode::FileNode(ref mut file_node) => {
                file_node.set_asset_id(asset_id.into())
            },
            WhichNode::IpAddressNode(ref mut ip_address_node) => {
                ip_address_node.set_asset_id(asset_id.into())
            },
            WhichNode::ProcessOutboundConnectionNode(ref mut process_outbound_connection_node) => {
                process_outbound_connection_node.set_asset_id(asset_id.into())
            },
            WhichNode::ProcessInboundConnectionNode(ref mut process_inbound_connection_node) => {
                process_inbound_connection_node.set_asset_id(asset_id.into())
            },
            WhichNode::IpPortNode(ref mut ip_port_node) => {
                ip_port_node.set_asset_id(asset_id.into())
            },
            WhichNode::NetworkConnectionNode(ref mut network_connection_node) => {
                network_connection_node.set_asset_id(asset_id.into())
            },
            WhichNode::IpConnectionNode(ip_connection_node) => {
                ip_connection_node.set_asset_id(asset_id.into())
            },
            WhichNode::DynamicNode(ref mut dynamic_node) => {
                dynamic_node.set_asset_id(asset_id.into())
            },
        }
    }

    fn get_node_key(&self) -> &str {
        let which_node = match self.which_node {
            Some(ref which_node) => which_node,
            None => {
                panic!("Failed to determine variant of node");
            }
        };

        match which_node {
            WhichNode::AssetNode(asset_node) => {
                asset_node.get_node_key()
            },
            WhichNode::ProcessNode(process_node) => {
                process_node.get_node_key()
            },
            WhichNode::FileNode(file_node) => {
                file_node.get_node_key()
            },
            WhichNode::IpAddressNode(ip_address_node) => {
                ip_address_node.get_node_key()
            },
            WhichNode::ProcessOutboundConnectionNode(process_outbound_connection_node) => {
                process_outbound_connection_node.get_node_key()
            },
            WhichNode::ProcessInboundConnectionNode(process_inbound_connection_node) => {
                process_inbound_connection_node.get_node_key()
            },
            WhichNode::IpPortNode(ip_port_node) => {
                ip_port_node.get_node_key()
            },
            WhichNode::NetworkConnectionNode(network_connection_node) => {
                network_connection_node.get_node_key()
            },
            WhichNode::IpConnectionNode(ip_connection_node) => {
                ip_connection_node.get_node_key()
            },
            WhichNode::DynamicNode(dynamic_node) => {
                dynamic_node.get_node_key()
            },
        }

    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => {
                warn!("Failed to determine variant of node");
                return;
            }
        };

        match which_node {
            WhichNode::AssetNode(ref mut asset_node) => {
                asset_node.set_node_key(node_key.into())
            },
            WhichNode::ProcessNode(ref mut process_node) => {
                process_node.set_node_key(node_key.into())
            },
            WhichNode::FileNode(ref mut file_node) => {
                file_node.set_node_key(node_key.into())
            },
            WhichNode::IpAddressNode(ref mut ip_address_node) => {
                ip_address_node.set_node_key(node_key.into())
            },
            WhichNode::ProcessOutboundConnectionNode(ref mut process_outbound_connection_node) => {
                process_outbound_connection_node.set_node_key(node_key.into())
            },
            WhichNode::ProcessInboundConnectionNode(ref mut process_inbound_connection_node) => {
                process_inbound_connection_node.set_node_key(node_key.into())
            },
            WhichNode::IpPortNode(ref mut ip_port_node) => {
                ip_port_node.set_node_key(node_key.into())
            },
            WhichNode::NetworkConnectionNode(ref mut network_connection_node) => {
                network_connection_node.set_node_key(node_key.into())
            },
            WhichNode::IpConnectionNode(ip_connection_node) => {
                ip_connection_node.set_node_key(node_key.into())
            },
            WhichNode::DynamicNode(ref mut dynamic_node) => {
                dynamic_node.set_node_key(node_key.into())
            },
        }
    }

    fn merge(&mut self, other: &Self) -> bool {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => {
                warn!("Failed to determine variant of node");
                return false;
            }
        };

        match which_node {
            WhichNode::AssetNode(ref mut asset_node) => {
                if let Some(WhichNode::AssetNode(ref other)) = other.which_node {
                    asset_node.merge(other)
                } else {
                    warn!("Attempted to merge AssetNode with non-AssetNode ");
                    false
                }
            },
            WhichNode::ProcessNode(ref mut process_node) => {
                if let Some(WhichNode::ProcessNode(ref other)) = other.which_node {
                    process_node.merge(other)
                } else {
                    warn!("Attempted to merge ProcessNode with non-ProcessNode ");
                    false
                }
            },
            WhichNode::FileNode(ref mut file_node) => {
                if let Some(WhichNode::FileNode(ref other)) = other.which_node {
                    file_node.merge(other)
                } else {
                    warn!("Attempted to merge FileNode with non-FileNode ");
                    false
                }
            },
            WhichNode::IpAddressNode(ref mut ip_address_node) => {
                if let Some(WhichNode::IpAddressNode(ref other)) = other.which_node {
                    ip_address_node.merge(other)
                } else {
                    warn!("Attempted to merge IpAddressNode with non-IpAddressNode ");
                    false
                }
            },
            WhichNode::ProcessOutboundConnectionNode(ref mut process_outbound_connection_node) => {
                if let Some(WhichNode::ProcessOutboundConnectionNode(ref other)) = other.which_node {
                    process_outbound_connection_node.merge(other)
                } else {
                    warn!("Attempted to merge ProcessOutboundConnectionNode with non-ProcessOutboundConnectionNode ");
                    false
                }
            },
            WhichNode::ProcessInboundConnectionNode(ref mut process_inbound_connection_node) => {
                if let Some(WhichNode::ProcessInboundConnectionNode(ref other)) = other.which_node {
                    process_inbound_connection_node.merge(other)
                } else {
                    warn!("Attempted to merge ProcessInboundConnectionNode with non-ProcessInboundConnectionNode ");
                    false
                }
            },
            WhichNode::IpPortNode(ref mut ip_port_node) => {
                if let Some(WhichNode::IpPortNode(ref other)) = other.which_node {
                    ip_port_node.merge(other)
                } else {
                    warn!("Attempted to merge IpPortNode with non-IpPortNode ");
                    false
                }
            },
            WhichNode::NetworkConnectionNode(ref mut network_connection_node) => {
                if let Some(WhichNode::NetworkConnectionNode(ref other)) = other.which_node {
                    network_connection_node.merge(other)
                } else {
                    warn!("Attempted to merge NetworkConnectionNode with non-NetworkConnectionNode ");
                    false
                }
            },
            WhichNode::IpConnectionNode(ip_connection_node) => {
                if let Some(WhichNode::IpConnectionNode(ref other)) = other.which_node {
                    ip_connection_node.merge(other)
                } else {
                    warn!("Attempted to merge IpConnectionNode with non-NetworkConnectionNode ");
                    false
                }
            },
            WhichNode::DynamicNode(ref mut dynamic_node) => {
                if let Some(WhichNode::DynamicNode(ref other)) = other.which_node {
                    dynamic_node.merge(other)
                } else {
                    warn!("Attempted to merge DynamicNode with non-DynamicNode ");
                    false
                }
            },
        }
    }

    fn merge_into(&mut self, other: Self) -> bool {
        let which_node = match self.which_node {
            Some(ref mut which_node) => which_node,
            None => {
                warn!("Failed to determine variant of node");
                return false;
            }
        };

        match which_node {
            WhichNode::AssetNode(ref mut asset_node) => {
                if let Some(WhichNode::AssetNode(other)) = other.which_node {
                    asset_node.merge_into(other)
                } else {
                    warn!("Attempted to merge AssetNode with non-AssetNode ");
                    false
                }
            },
            WhichNode::ProcessNode(ref mut process_node) => {
                if let Some(WhichNode::ProcessNode(other)) = other.which_node {
                    process_node.merge_into(other)
                } else {
                    warn!("Attempted to merge ProcessNode with non-ProcessNode ");
                    false
                }
            },
            WhichNode::FileNode(ref mut file_node) => {
                if let Some(WhichNode::FileNode(other)) = other.which_node {
                    file_node.merge_into(other)
                } else {
                    warn!("Attempted to merge FileNode with non-FileNode ");
                    false
                }
            },
            WhichNode::IpAddressNode(ref mut ip_address_node) => {
                if let Some(WhichNode::IpAddressNode(other)) = other.which_node {
                    ip_address_node.merge_into(other)
                } else {
                    warn!("Attempted to merge IpAddressNode with non-IpAddressNode ");
                    false
                }
            },
            WhichNode::ProcessOutboundConnectionNode(ref mut process_outbound_connection_node) => {
                if let Some(WhichNode::ProcessOutboundConnectionNode(other)) = other.which_node {
                    process_outbound_connection_node.merge_into(other)
                } else {
                    warn!("Attempted to merge ProcessOutboundConnectionNode with non-ProcessOutboundConnectionNode ");
                    false
                }
            },
            WhichNode::ProcessInboundConnectionNode(ref mut process_inbound_connection_node) => {
                if let Some(WhichNode::ProcessInboundConnectionNode(other)) = other.which_node {
                    process_inbound_connection_node.merge_into(other)
                } else {
                    warn!("Attempted to merge ProcessInboundConnectionNode with non-ProcessInboundConnectionNode ");
                    false
                }
            },
            WhichNode::IpPortNode(ref mut ip_port_node) => {
                if let Some(WhichNode::IpPortNode(other)) = other.which_node {
                    ip_port_node.merge_into(other)
                } else {
                    warn!("Attempted to merge IpPortNode with non-IpPortNode ");
                    false
                }
            },
            WhichNode::NetworkConnectionNode(ref mut network_connection_node) => {
                if let Some(WhichNode::NetworkConnectionNode(other)) = other.which_node {
                    network_connection_node.merge_into(other)
                } else {
                    warn!("Attempted to merge NetworkConnectionNode with non-NetworkConnectionNode ");
                    false
                }
            },
            WhichNode::IpConnectionNode(ip_connection_node) => {
                if let Some(WhichNode::IpConnectionNode(other)) = other.which_node {
                    ip_connection_node.merge_into(other)
                } else {
                    warn!("Attempted to merge IpConnectionNode with non-NetworkConnectionNode ");
                    false
                }
            },
            WhichNode::DynamicNode(ref mut dynamic_node) => {
                if let Some(WhichNode::DynamicNode(other)) = other.which_node {
                    dynamic_node.merge_into(other)
                } else {
                    warn!("Attempted to merge DynamicNode with non-DynamicNode ");
                    false
                }
            },
        }

    }
}
