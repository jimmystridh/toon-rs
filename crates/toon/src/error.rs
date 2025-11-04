use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[cfg(feature = "serde")]
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("syntax at line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;
