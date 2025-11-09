/// Error types
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Io error
    #[error("io error")]
    Io(#[from] std::io::Error),

    /// Serde error
    #[error("serde error")]
    Serde(#[from] serde_json::Error),

    /// Unexpected command type
    #[error("unexpected cmd")]
    UnexpectedCommandType,

    /// Key not found
    #[error("key not found")]
    KeyNotFound,
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
