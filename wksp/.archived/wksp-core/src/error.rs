use thiserror::Error;
use serde::{ Deserialize, Serialize };

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum WkspError {
    #[error("API Error: {0}")] Api(String),

    #[error("Configuration Error: {0}")] Config(String),

    #[error("Internal Error: {0}")] Internal(String),

    #[error("Not Found: {0}")] NotFound(String),

    #[error("Authentication Error: {0}")] Auth(String),
}

pub type Result<T> = std::result::Result<T, WkspError>;
