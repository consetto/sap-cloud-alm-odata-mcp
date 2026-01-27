//! Analytics API client (OData v4) - CALM_ANALYTICS_ODATA.

use serde_json::Value;

use crate::error::ApiError;
use crate::odata::{ODataClient, ODataQuery};

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

    /// Get defects analytics.
    pub async fn get_defects(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Defects", query).await
    }

    /// Get features analytics.
    pub async fn get_features(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Features", query).await
    }

    /// Get tests analytics.
    pub async fn get_tests(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Tests", query).await
    }

    /// Get quality gates analytics.
    pub async fn get_quality_gates(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/QualityGates", query).await
    }

    /// Get projects analytics.
    pub async fn get_projects(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Projects", query).await
    }

    /// Get configuration items analytics.
    pub async fn get_configuration_items(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/ConfigurationItems", query).await
    }

    /// Get exceptions analytics.
    pub async fn get_exceptions(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Exceptions", query).await
    }

    /// Get jobs analytics.
    pub async fn get_jobs(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Jobs", query).await
    }

    /// Get messages analytics.
    pub async fn get_messages(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Messages", query).await
    }

    /// Get metrics analytics.
    pub async fn get_metrics(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Metrics", query).await
    }

    /// Get monitoring events analytics.
    pub async fn get_monitoring_events(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/MonitoringEvents", query).await
    }

    /// Get requests analytics.
    pub async fn get_requests(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Requests", query).await
    }

    /// Get scenario executions analytics.
    pub async fn get_scenario_executions(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/ScenarioExecutions", query).await
    }

    /// Get service levels analytics.
    pub async fn get_service_levels(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/ServiceLevels", query).await
    }

    /// Get status events analytics.
    pub async fn get_status_events(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/StatusEvents", query).await
    }
}

impl std::fmt::Debug for AnalyticsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyticsClient").finish()
    }
}
