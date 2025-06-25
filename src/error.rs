use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LCoreError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Device authentication error: {0}")]
    DeviceAuth(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

// Legacy AppError for backward compatibility
#[derive(Debug)]
pub enum AppError {
    Internal(String),
    InvalidInput(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<LCoreError> for AppError {
    fn from(err: LCoreError) -> Self {
        AppError::Internal(err.to_string())
    }
} 