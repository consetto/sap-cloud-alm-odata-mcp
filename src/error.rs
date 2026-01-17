//! Unified error types for the SAP Cloud ALM MCP Server.

use reqwest::StatusCode;
use thiserror::Error;

/// Main application error type.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("MCP error: {0}")]
    Mcp(String),
}

/// Configuration-related errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

/// Authentication-related errors.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Token request failed with status {status}: {body}")]
    TokenRequestFailed { status: StatusCode, body: String },

    #[error("Token parse error: {0}")]
    TokenParse(String),

    #[error("Token expired and refresh failed")]
    TokenExpired,

    #[error("No token available")]
    NoToken,
}

/// API request/response errors.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("HTTP error {status}: {body}")]
    HttpError { status: StatusCode, body: String },

    #[error("OData error [{code}]: {message}")]
    ODataError {
        status: StatusCode,
        code: String,
        message: String,
    },

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Request timeout")]
    Timeout,

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

impl ApiError {
    /// Create an HTTP error from status and body.
    pub fn http_error(status: StatusCode, body: String) -> Self {
        ApiError::HttpError { status, body }
    }

    /// Create an OData error.
    pub fn odata_error(status: StatusCode, code: String, message: String) -> Self {
        ApiError::ODataError {
            status,
            code,
            message,
        }
    }

    /// Check if this is a "not found" error (404).
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            ApiError::HttpError { status, .. } if *status == StatusCode::NOT_FOUND
        ) || matches!(self, ApiError::NotFound(_))
    }

    /// Check if this is an authentication error (401/403).
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            ApiError::HttpError { status, .. }
                if *status == StatusCode::UNAUTHORIZED || *status == StatusCode::FORBIDDEN
        ) || matches!(self, ApiError::Auth(_))
    }
}
