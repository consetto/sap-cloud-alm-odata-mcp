//! Configuration management for SAP Cloud ALM MCP Server.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::ConfigError;

/// Sandbox API base URL for SAP Cloud ALM.
const SANDBOX_BASE_URL: &str = "https://sandbox.api.sap.com/SAPCALM";

/// Main configuration structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Sandbox mode - use static API key instead of OAuth2
    #[serde(default)]
    pub sandbox: bool,

    /// API key for sandbox mode (required when sandbox=true)
    pub api_key: Option<String>,

    /// SAP Cloud ALM tenant identifier (e.g., "my-company-calm")
    /// Required in OAuth2 mode, ignored in sandbox mode.
    pub tenant: Option<String>,

    /// SAP Cloud ALM region (e.g., "eu10", "us10", "ap10")
    /// Required in OAuth2 mode, ignored in sandbox mode.
    pub region: Option<String>,

    /// OAuth2 client ID from service binding
    /// Required in OAuth2 mode, ignored in sandbox mode.
    pub client_id: Option<String>,

    /// OAuth2 client secret from service binding
    /// Required in OAuth2 mode, ignored in sandbox mode.
    pub client_secret: Option<String>,

    /// Enable debug mode for MCP message logging
    #[serde(default)]
    pub debug: bool,

    /// HTTP request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Buffer before token expiration to refresh (seconds)
    #[serde(default = "default_token_buffer")]
    pub token_refresh_buffer_seconds: u64,
}

fn default_timeout() -> u64 {
    30
}

fn default_token_buffer() -> u64 {
    5
}

impl Config {
    /// Load configuration from a file path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values.
    fn validate(&self) -> Result<(), ConfigError> {
        if self.sandbox {
            // Sandbox mode: require api_key
            match &self.api_key {
                None => {
                    return Err(ConfigError::MissingField(
                        "api_key (required in sandbox mode)".into(),
                    ))
                }
                Some(key) if key.is_empty() => {
                    return Err(ConfigError::MissingField(
                        "api_key (required in sandbox mode)".into(),
                    ))
                }
                _ => {}
            }
        } else {
            // OAuth2 mode: require tenant, region, client_id, client_secret
            match &self.tenant {
                None => return Err(ConfigError::MissingField("tenant".into())),
                Some(t) if t.is_empty() => return Err(ConfigError::MissingField("tenant".into())),
                _ => {}
            }
            match &self.region {
                None => return Err(ConfigError::MissingField("region".into())),
                Some(r) if r.is_empty() => return Err(ConfigError::MissingField("region".into())),
                _ => {}
            }
            match &self.client_id {
                None => return Err(ConfigError::MissingField("client_id".into())),
                Some(c) if c.is_empty() => {
                    return Err(ConfigError::MissingField("client_id".into()))
                }
                _ => {}
            }
            match &self.client_secret {
                None => return Err(ConfigError::MissingField("client_secret".into())),
                Some(s) if s.is_empty() => {
                    return Err(ConfigError::MissingField("client_secret".into()))
                }
                _ => {}
            }

            // Validate region is one of the known values
            let valid_regions = [
                "eu10", "eu20", "us10", "ap10", "jp10", "eu10-004", "ca10", "eu11", "cn20",
            ];
            let region = self
                .region
                .as_ref()
                .expect("region already validated as present");
            if !valid_regions.contains(&region.as_str()) {
                return Err(ConfigError::Invalid(format!(
                    "Invalid region '{}'. Valid regions: {:?}",
                    region, valid_regions
                )));
            }
        }

        Ok(())
    }

    /// Get the OAuth2 token URL.
    /// Returns None in sandbox mode.
    ///
    /// # Panics
    /// Panics if called in OAuth2 mode without tenant/region being set.
    /// This should not happen if config was validated via `validate()`.
    pub fn token_url(&self) -> Option<String> {
        if self.sandbox {
            None
        } else {
            Some(format!(
                "https://{}.authentication.{}.hana.ondemand.com/oauth/token",
                self.tenant
                    .as_ref()
                    .expect("tenant required in OAuth2 mode"),
                self.region
                    .as_ref()
                    .expect("region required in OAuth2 mode")
            ))
        }
    }

    /// Get the API base URL.
    ///
    /// # Panics
    /// Panics if called in OAuth2 mode without tenant/region being set.
    /// This should not happen if config was validated via `validate()`.
    pub fn api_base_url(&self) -> String {
        if self.sandbox {
            SANDBOX_BASE_URL.to_string()
        } else {
            format!(
                "https://{}.{}.alm.cloud.sap",
                self.tenant
                    .as_ref()
                    .expect("tenant required in OAuth2 mode"),
                self.region
                    .as_ref()
                    .expect("region required in OAuth2 mode")
            )
        }
    }

    /// Get the API path prefix.
    /// Sandbox mode uses direct paths, OAuth2 mode uses /api prefix.
    fn api_path_prefix(&self) -> &'static str {
        if self.sandbox {
            ""
        } else {
            "/api"
        }
    }

    /// Get the Features API URL.
    pub fn features_api_url(&self) -> String {
        format!(
            "{}{}/calm-features/v1",
            self.api_base_url(),
            self.api_path_prefix()
        )
    }

    /// Get the Documents API URL.
    pub fn documents_api_url(&self) -> String {
        format!(
            "{}{}/calm-documents/v1",
            self.api_base_url(),
            self.api_path_prefix()
        )
    }

    /// Get the Tasks API URL.
    pub fn tasks_api_url(&self) -> String {
        format!(
            "{}{}/calm-tasks/v1",
            self.api_base_url(),
            self.api_path_prefix()
        )
    }

    /// Get the Projects API URL.
    pub fn projects_api_url(&self) -> String {
        format!(
            "{}{}/calm-projects/v1",
            self.api_base_url(),
            self.api_path_prefix()
        )
    }

    /// Get the Test Management API URL.
    pub fn testmanagement_api_url(&self) -> String {
        format!(
            "{}{}/calm-testmanagement/v1",
            self.api_base_url(),
            self.api_path_prefix()
        )
    }

    /// Get the Process Hierarchy API URL.
    pub fn processhierarchy_api_url(&self) -> String {
        format!(
            "{}{}/calm-processhierarchy/v1",
            self.api_base_url(),
            self.api_path_prefix()
        )
    }

    /// Get the Analytics API URL.
    pub fn analytics_api_url(&self) -> String {
        format!(
            "{}{}/calm-analytics/v1/odata/v4/analytics",
            self.api_base_url(),
            self.api_path_prefix()
        )
    }

    /// Get the Process Monitoring API URL.
    pub fn processmonitoring_api_url(&self) -> String {
        format!(
            "{}{}/calm-processmonitoring/v1",
            self.api_base_url(),
            self.api_path_prefix()
        )
    }

    /// Get the Logs API URL.
    pub fn logs_api_url(&self) -> String {
        format!(
            "{}{}/calm-logs/v1",
            self.api_base_url(),
            self.api_path_prefix()
        )
    }

    /// Get timeout as Duration.
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }

    /// Get token refresh buffer as chrono Duration.
    pub fn token_buffer(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.token_refresh_buffer_seconds as i64)
    }

    /// Check if running in sandbox mode.
    #[cfg(test)]
    pub fn is_sandbox(&self) -> bool {
        self.sandbox
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_url_construction() {
        let config = Config {
            sandbox: false,
            api_key: None,
            tenant: Some("mycompany".to_string()),
            region: Some("eu10".to_string()),
            client_id: Some("test-client".to_string()),
            client_secret: Some("test-secret".to_string()),
            debug: false,
            timeout_seconds: 30,
            token_refresh_buffer_seconds: 5,
        };

        assert_eq!(
            config.token_url(),
            Some("https://mycompany.authentication.eu10.hana.ondemand.com/oauth/token".to_string())
        );
        assert_eq!(
            config.api_base_url(),
            "https://mycompany.eu10.alm.cloud.sap"
        );
        assert_eq!(
            config.features_api_url(),
            "https://mycompany.eu10.alm.cloud.sap/api/calm-features/v1"
        );
        assert!(!config.is_sandbox());
    }

    #[test]
    fn test_sandbox_url_construction() {
        let config = Config {
            sandbox: true,
            api_key: Some("test-api-key".to_string()),
            tenant: None,
            region: None,
            client_id: None,
            client_secret: None,
            debug: true,
            timeout_seconds: 30,
            token_refresh_buffer_seconds: 5,
        };

        assert_eq!(config.token_url(), None);
        assert_eq!(config.api_base_url(), "https://sandbox.api.sap.com/SAPCALM");
        assert_eq!(
            config.features_api_url(),
            "https://sandbox.api.sap.com/SAPCALM/calm-features/v1"
        );
        assert!(config.is_sandbox());
    }
}
