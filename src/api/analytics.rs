//! Analytics API client (OData v4) - CALM_ANALYTICS_ODATA.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ApiError;
use crate::odata::{ODataClient, ODataQuery};

/// Analytics data provider info.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataProvider {
    pub name: Option<String>,
    pub description: Option<String>,
    pub dimensions: Option<Vec<String>>,
    pub metrics: Option<Vec<String>>,
}

/// Analytics API client.
#[derive(Clone)]
pub struct AnalyticsClient {
    odata_client: ODataClient,
}

impl AnalyticsClient {
    pub fn new(odata_client: ODataClient) -> Self {
        Self { odata_client }
    }

    /// Query a generic dataset by provider name.
    pub async fn query_dataset(&self, provider: &str, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        let endpoint = format!("/DataSet('{}')", provider);
        self.odata_client.get_collection_raw(&endpoint, query).await
    }

    /// Get requirements analytics.
    pub async fn get_requirements(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Requirements", query).await
    }

    /// Get tasks analytics.
    pub async fn get_tasks_analytics(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Tasks", query).await
    }

    /// Get alerts analytics.
    pub async fn get_alerts(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Alerts", query).await
    }

    /// List available providers.
    pub async fn list_providers(&self) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Providers", None).await
    }
}

impl std::fmt::Debug for AnalyticsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyticsClient").finish()
    }
}
