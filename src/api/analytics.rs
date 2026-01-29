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
    /// The provider is passed as a $filter parameter: provider eq 'ProviderName'
    pub async fn query_dataset(
        &self,
        provider: &str,
        additional_filter: Option<String>,
        top: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Value, ApiError> {
        let provider_filter = format!("provider eq '{}'", provider);

        // Combine provider filter with any additional filter
        let full_filter = match additional_filter {
            Some(existing) => format!("{} and {}", provider_filter, existing),
            None => provider_filter,
        };

        let mut query = ODataQuery::new().filter(full_filter);

        if let Some(t) = top {
            query = query.top(t);
        }
        if let Some(s) = skip {
            query = query.skip(s);
        }

        self.odata_client
            .get_collection_raw("/DataSet", Some(query))
            .await
    }

    /// Get requirements analytics.
    pub async fn get_requirements(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/Requirements", query)
            .await
    }

    /// Get tasks analytics.
    pub async fn get_tasks_analytics(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Tasks", query).await
    }

    /// List available providers (static list based on available entity sets).
    pub fn list_providers(&self) -> Value {
        serde_json::json!({
            "providers": [
                {"name": "Requirements", "description": "Requirements analytics data"},
                {"name": "Projects", "description": "Projects analytics data"},
                {"name": "Tasks", "description": "Tasks analytics data"},
                {"name": "Defects", "description": "Defects analytics data"},
                {"name": "Tests", "description": "Tests analytics data"},
                {"name": "Features", "description": "Features analytics data"},
                {"name": "ConfigurationItems", "description": "Configuration items analytics data"},
                {"name": "Metrics", "description": "Metrics analytics data"},
                {"name": "Requests", "description": "Requests analytics data"},
                {"name": "Exceptions", "description": "Exceptions analytics data"},
                {"name": "StatusEvents", "description": "Status events analytics data"},
                {"name": "QualityGates", "description": "Quality gates analytics data"},
                {"name": "Jobs", "description": "Jobs analytics data"},
                {"name": "ServiceLevels", "description": "Service levels analytics data"},
                {"name": "ScenarioExecutions", "description": "Scenario executions analytics data"},
                {"name": "MonitoringEvents", "description": "Monitoring events analytics data"},
                {"name": "Messages", "description": "Messages analytics data"}
            ],
            "note": "Use these provider names with query_analytics_dataset or the dedicated get_analytics_* tools."
        })
    }

    /// Get defects analytics.
    pub async fn get_defects(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/Defects", query)
            .await
    }

    /// Get features analytics.
    pub async fn get_features(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/Features", query)
            .await
    }

    /// Get tests analytics.
    pub async fn get_tests(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Tests", query).await
    }

    /// Get quality gates analytics.
    pub async fn get_quality_gates(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/QualityGates", query)
            .await
    }

    /// Get projects analytics.
    pub async fn get_projects(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/Projects", query)
            .await
    }

    /// Get configuration items analytics.
    pub async fn get_configuration_items(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/ConfigurationItems", query)
            .await
    }

    /// Get exceptions analytics.
    pub async fn get_exceptions(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/Exceptions", query)
            .await
    }

    /// Get jobs analytics.
    pub async fn get_jobs(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Jobs", query).await
    }

    /// Get messages analytics.
    pub async fn get_messages(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/Messages", query)
            .await
    }

    /// Get metrics analytics.
    pub async fn get_metrics(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/Metrics", query)
            .await
    }

    /// Get monitoring events analytics.
    pub async fn get_monitoring_events(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/MonitoringEvents", query)
            .await
    }

    /// Get requests analytics.
    pub async fn get_requests(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/Requests", query)
            .await
    }

    /// Get scenario executions analytics.
    pub async fn get_scenario_executions(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/ScenarioExecutions", query)
            .await
    }

    /// Get service levels analytics.
    pub async fn get_service_levels(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/ServiceLevels", query)
            .await
    }

    /// Get status events analytics.
    pub async fn get_status_events(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/StatusEvents", query)
            .await
    }
}

impl std::fmt::Debug for AnalyticsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyticsClient").finish()
    }
}
