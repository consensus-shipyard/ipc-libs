use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Custom error with string as the detailed message
    #[error("Custom error: {0}")]
    Custom(String),
}
