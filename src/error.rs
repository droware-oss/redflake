use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Unsupported protocol version: {0} (supported: 2, 3)")]
    UnsupportedProtocol(u8),
}
