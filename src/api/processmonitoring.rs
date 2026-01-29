//! Process Monitoring API client (OData v4) - CALM_PMGE.

use serde_json::Value;

use crate::error::ApiError;
use crate::odata::{ODataClient, ODataQuery};

/// Process Monitoring API client.
#[derive(Clone)]
pub struct ProcessMonitoringClient {
    odata_client: ODataClient,
}

impl ProcessMonitoringClient {
    pub fn new(odata_client: ODataClient) -> Self {
        Self { odata_client }
    }

    /// List business processes.
    pub async fn list_business_processes(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/businessProcesses", query)
            .await
    }

    /// Get a business process by ID.
    pub async fn get_business_process(&self, id: &str) -> Result<Value, ApiError> {
        self.odata_client
            .get_entity_by_uuid::<Value>("/businessProcesses", id)
            .await
    }

    /// List solution processes.
    pub async fn list_solution_processes(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/solutionProcesses", query)
            .await
    }

    /// Get a solution process by ID.
    pub async fn get_solution_process(&self, id: &str) -> Result<Value, ApiError> {
        self.odata_client
            .get_entity_by_uuid::<Value>("/solutionProcesses", id)
            .await
    }

    /// List solution process flows.
    pub async fn list_solution_process_flows(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/solutionProcessFlows", query)
            .await
    }

    /// List solution value flow diagrams.
    pub async fn list_solution_value_flow_diagrams(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_collection_raw("/solutionValueFlowDiagrams", query)
            .await
    }

    /// List assets.
    pub async fn list_assets(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/assets", query).await
    }
}

impl std::fmt::Debug for ProcessMonitoringClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessMonitoringClient").finish()
    }
}
