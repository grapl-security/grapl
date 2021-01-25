pub mod asset;
pub mod error;
pub mod file;
pub mod graph;
pub mod ip_address;
pub mod ip_connection;
pub mod ip_port;
pub mod network_connection;
pub mod process;
pub mod process_inbound_connection;
pub mod process_outbound_connection;

pub use crate::{asset::{Asset,
                        AssetNode,
                        IAssetNode},
                error::Error,
                file::{File,
                       FileNode,
                       IFileNode},
                ip_address::{IIpAddressNode,
                             IpAddress,
                             IpAddressNode},
                ip_connection::{IIpConnectionNode,
                                IpConnection,
                                IpConnectionNode},
                ip_port::{IIpPortNode,
                          IpPort,
                          IpPortNode},
                network_connection::{INetworkConnectionNode,
                                     NetworkConnection,
                                     NetworkConnectionNode},
                process::{IProcessNode,
                          Process,
                          ProcessNode},
                process_inbound_connection::{IProcessInboundConnectionNode,
                                             ProcessInboundConnection,
                                             ProcessInboundConnectionNode},
                process_outbound_connection::{IProcessOutboundConnectionNode,
                                              ProcessOutboundConnection,
                                              ProcessOutboundConnectionNode}};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
