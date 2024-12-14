use thiserror::Error;

#[derive(Error, Debug)]
pub enum KanaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to serialize/deserialize data: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, KanaError>;