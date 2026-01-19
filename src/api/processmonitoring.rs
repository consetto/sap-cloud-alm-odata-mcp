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

    pub async fn list_events(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Events", query).await
    }

    pub async fn get_event(&self, id: &str) -> Result<Value, ApiError> {
        self.odata_client.get_entity_by_uuid::<Value>("/Events", id).await
    }

    pub async fn list_services(&self, query: Option<ODataQuery>) -> Result<Value, ApiError> {
        self.odata_client.get_collection_raw("/Services", query).await
    }
}

impl std::fmt::Debug for ProcessMonitoringClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessMonitoringClient").finish()
    }
}
