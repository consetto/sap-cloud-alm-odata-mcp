//! OAuth2 client credentials authentication for SAP Cloud ALM.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::error::AuthError;

/// OAuth2 token response from SAP.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    expires_in: i64,
    #[allow(dead_code)]
    #[serde(default)]
    scope: String,
}

/// Cached token with expiration tracking.
#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: DateTime<Utc>,
}

impl CachedToken {
    /// Check if token is expired (with buffer).
    fn is_expired(&self, buffer: Duration) -> bool {
        Utc::now() + buffer >= self.expires_at
    }
}

/// OAuth2 client for SAP Cloud ALM authentication.
/// Also supports sandbox mode with static API key.
#[derive(Clone)]
pub struct OAuth2Client {
    config: Config,
    http_client: Client,
    token_cache: Arc<RwLock<Option<CachedToken>>>,
}

impl OAuth2Client {
    /// Create a new OAuth2 client.
    pub fn new(config: Config) -> Self {
        let http_client = Client::builder()
            .timeout(config.timeout())
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Get a valid access token, refreshing if necessary.
    /// In sandbox mode, returns the static API key directly.
    pub async fn get_token(&self) -> Result<String, AuthError> {
        // If sandbox mode, return API key directly
        if self.config.sandbox {
            return self.config.api_key.clone()
                .ok_or(AuthError::NoToken);
        }

        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if let Some(ref cached) = *cache {
                if !cached.is_expired(self.config.token_buffer()) {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Fetch new token
        self.fetch_token().await
    }

    /// Check if running in sandbox mode.
    pub fn is_sandbox(&self) -> bool {
        self.config.sandbox
    }

    /// Fetch a new token from the OAuth2 token endpoint.
    async fn fetch_token(&self) -> Result<String, AuthError> {
        let token_url = self.config.token_url()
            .ok_or_else(|| AuthError::TokenParse("No token URL in sandbox mode".to_string()))?;

        // Create Basic Auth header (Base64 encoded client_id:client_secret)
        let client_id = self.config.client_id.as_ref()
            .ok_or_else(|| AuthError::TokenParse("Missing client_id".to_string()))?;
        let client_secret = self.config.client_secret.as_ref()
            .ok_or_else(|| AuthError::TokenParse("Missing client_secret".to_string()))?;
        let credentials = format!("{}:{}", client_id, client_secret);
        let encoded = BASE64.encode(credentials.as_bytes());
        let auth_header = format!("Basic {}", encoded);

        if self.config.debug {
            eprintln!("[AUTH] Fetching token from: {}", token_url);
        }

        let response = self
            .http_client
            .post(&token_url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if self.config.debug {
                eprintln!("[AUTH] Token request failed: {} - {}", status, body);
            }
            return Err(AuthError::TokenRequestFailed { status, body });
        }

        let token_response: TokenResponse = response.json().await.map_err(|e| {
            AuthError::TokenParse(format!("Failed to parse token response: {}", e))
        })?;

        // Calculate expiration time
        let expires_at = Utc::now() + Duration::seconds(token_response.expires_in);

        if self.config.debug {
            eprintln!(
                "[AUTH] Token acquired, expires at: {}",
                expires_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }

        // Cache the token
        let cached = CachedToken {
            access_token: token_response.access_token.clone(),
            expires_at,
        };

        {
            let mut cache = self.token_cache.write().await;
            *cache = Some(cached);
        }

        Ok(token_response.access_token)
    }

}

impl std::fmt::Debug for OAuth2Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.config.sandbox {
            f.debug_struct("OAuth2Client")
                .field("mode", &"sandbox")
                .finish()
        } else {
            f.debug_struct("OAuth2Client")
                .field("tenant", &self.config.tenant)
                .field("region", &self.config.region)
                .finish()
        }
    }
}
