//! Unified error types for the SAP Cloud ALM MCP Server.

use reqwest::StatusCode;
use thiserror::Error;

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

    #[error("No token available")]
    NoToken,

    #[error("Failed to create HTTP client: {0}")]
    HttpClientInit(String),
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

    #[error("Failed to create HTTP client: {0}")]
    HttpClientInit(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_missing_field_display() {
        let error = ConfigError::MissingField("tenant".to_string());
        assert_eq!(error.to_string(), "Missing required field: tenant");
    }

    #[test]
    fn test_config_error_invalid_display() {
        let error = ConfigError::Invalid("region must be valid".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid configuration: region must be valid"
        );
    }

    #[test]
    fn test_config_error_io_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_err: ConfigError = io_err.into();
        assert!(config_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_auth_error_no_token_display() {
        let error = AuthError::NoToken;
        assert_eq!(error.to_string(), "No token available");
    }

    #[test]
    fn test_auth_error_token_parse_display() {
        let error = AuthError::TokenParse("invalid JSON".to_string());
        assert_eq!(error.to_string(), "Token parse error: invalid JSON");
    }

    #[test]
    fn test_auth_error_http_client_init_display() {
        let error = AuthError::HttpClientInit("connection timeout".to_string());
        assert_eq!(
            error.to_string(),
            "Failed to create HTTP client: connection timeout"
        );
    }

    #[test]
    fn test_auth_error_token_request_failed_display() {
        let error = AuthError::TokenRequestFailed {
            status: StatusCode::UNAUTHORIZED,
            body: "Invalid credentials".to_string(),
        };
        let display = error.to_string();
        assert!(display.contains("401"));
        assert!(display.contains("Invalid credentials"));
    }

    #[test]
    fn test_api_error_http_error_display() {
        let error = ApiError::HttpError {
            status: StatusCode::NOT_FOUND,
            body: "Resource not found".to_string(),
        };
        let display = error.to_string();
        assert!(display.contains("404"));
        assert!(display.contains("Resource not found"));
    }

    #[test]
    fn test_api_error_odata_error_display() {
        let error = ApiError::ODataError {
            status: StatusCode::BAD_REQUEST,
            code: "INVALID_INPUT".to_string(),
            message: "Field 'title' is required".to_string(),
        };
        let display = error.to_string();
        assert!(display.contains("INVALID_INPUT"));
        assert!(display.contains("Field 'title' is required"));
    }

    #[test]
    fn test_api_error_http_client_init_display() {
        let error = ApiError::HttpClientInit("TLS error".to_string());
        assert_eq!(error.to_string(), "Failed to create HTTP client: TLS error");
    }

    #[test]
    fn test_api_error_from_auth_error() {
        let auth_error = AuthError::NoToken;
        let api_error: ApiError = auth_error.into();
        assert!(api_error.to_string().contains("Authentication error"));
    }

    #[test]
    fn test_config_error_debug_format() {
        let error = ConfigError::MissingField("api_key".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("MissingField"));
        assert!(debug.contains("api_key"));
    }

    #[test]
    fn test_auth_error_debug_format() {
        let error = AuthError::TokenParse("unexpected token".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("TokenParse"));
    }

    #[test]
    fn test_api_error_debug_format() {
        let error = ApiError::ODataError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "ERR500".to_string(),
            message: "Internal error".to_string(),
        };
        let debug = format!("{:?}", error);
        assert!(debug.contains("ODataError"));
        assert!(debug.contains("ERR500"));
    }
}
