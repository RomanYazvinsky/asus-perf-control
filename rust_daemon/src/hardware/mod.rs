pub mod cpu_boost;
pub mod fan;
pub mod temp;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HardwareError {
    #[error("Failed to read from {path}: {source}")]
    ReadError {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to write to {path}: {source}")]
    WriteError {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to parse value from {path}: {source}")]
    ParseError {
        path: String,
        source: std::num::ParseIntError,
    },
}
