use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("InvalidProcessState: {0}")]
    InvalidProcessState(u32),
    #[error("InvalidProcessOutboundConnectionState: {0}")]
    InvalidProcessOutboundConnectionState(u32),
    #[error("InvalidProcessInboundConnectionState: {0}")]
    InvalidProcessInboundConnectionState(u32),
    #[error("InvalidFileState: {0}")]
    InvalidFileState(u32),
    #[error("InvalidNetworkConnectionState: {0}")]
    InvalidNetworkConnectionState(u32),
    #[error("InvalidIpConnectionState: {0}")]
    InvalidIpConnectionState(u32),
}
