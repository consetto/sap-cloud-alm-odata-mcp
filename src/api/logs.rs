//! Logs API client (REST) - CALM_LOGS.
//! OpenTelemetry format for log records.

use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use crate::auth::OAuth2Client;
use crate::error::ApiError;

/// Query parameters for getting logs.
#[derive(Debug, Clone, Default)]
pub struct GetLogsParams {
    pub provider: String,
    pub format: Option<String>,
    pub version: Option<String>,
    pub period: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub service_id: Option<String>,
    pub observed_timestamp: Option<bool>,
    pub on_limit: Option<String>,
}

/// Parameters for posting logs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostLogsParams {
    pub use_case: String,
    pub service_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Logs API client.
#[derive(Clone)]
pub struct LogsClient {
    base_url: String,
    http_client: Client,
    auth_client: OAuth2Client,
    debug: bool,
    is_sandbox: bool,
}

impl LogsClient {
    pub fn new(base_url: String, auth_client: OAuth2Client, debug: bool) -> Self {
        let is_sandbox = auth_client.is_sandbox();
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            http_client,
            auth_client,
            debug,
            is_sandbox,
        }
    }

    /// Get the appropriate auth header name and value.
    fn auth_header(&self, token: &str) -> (&'static str, String) {
        if self.is_sandbox {
            ("APIKey", token.to_string())
        } else {
            ("Authorization", format!("Bearer {}", token))
        }
    }

    /// Get logs (outbound).
    pub async fn get_logs(&self, params: &GetLogsParams) -> Result<Value, ApiError> {
        let mut url = format!("{}/logs?provider={}", self.base_url, params.provider);

        if let Some(ref f) = params.format {
            url.push_str(&format!("&format={}", f));
        }
        if let Some(ref v) = params.version {
            url.push_str(&format!("&version={}", v));
        }
        if let Some(ref p) = params.period {
            url.push_str(&format!("&period={}", p));
        }
        if let Some(ref from) = params.from {
            url.push_str(&format!("&from={}", from));
        }
        if let Some(ref to) = params.to {
            url.push_str(&format!("&to={}", to));
        }
        if let Some(limit) = params.limit {
            url.push_str(&format!("&limit={}", limit));
        }
        if let Some(offset) = params.offset {
            url.push_str(&format!("&offset={}", offset));
        }
        if let Some(ref sid) = params.service_id {
            url.push_str(&format!("&logsFilters[serviceId]={}", sid));
        }
        if let Some(ot) = params.observed_timestamp {
            url.push_str(&format!("&observedTimestamp={}", ot));
        }
        if let Some(ref ol) = params.on_limit {
            url.push_str(&format!("&onLimit={}", ol));
        }

        self.get(&url).await
    }

    /// Post logs (inbound).
    pub async fn post_logs(&self, params: &PostLogsParams, logs: &Value) -> Result<Value, ApiError> {
        let mut url = format!(
            "{}/logs?useCase={}&serviceId={}",
            self.base_url, params.use_case, params.service_id
        );

        if let Some(ref v) = params.version {
            url.push_str(&format!("&version={}", v));
        }
        if let Some(d) = params.dev {
            url.push_str(&format!("&dev={}", d));
        }
        if let Some(ref t) = params.tag {
            url.push_str(&format!("&tag={}", urlencoding::encode(t)));
        }

        self.post(&url, logs).await
    }

    async fn get(&self, url: &str) -> Result<Value, ApiError> {
        if self.debug {
            eprintln!("[LOGS] GET {}", url);
        }

        let token = self.auth_client.get_token().await?;
        let (header_name, header_value) = self.auth_header(&token);

        let response = self
            .http_client
            .get(url)
            .header(header_name, header_value)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::HttpError { status, body })
        }
    }

    async fn post(&self, url: &str, body: &Value) -> Result<Value, ApiError> {
        if self.debug {
            eprintln!("[LOGS] POST {}", url);
        }

        let token = self.auth_client.get_token().await?;
        let (header_name, header_value) = self.auth_header(&token);

        let response = self
            .http_client
            .post(url)
            .header(header_name, header_value)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await.unwrap_or(Value::Null))
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::HttpError { status, body })
        }
    }
}

impl std::fmt::Debug for LogsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogsClient")
            .field("base_url", &self.base_url)
            .finish()
    }
}
