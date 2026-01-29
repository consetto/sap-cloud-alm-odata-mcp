//! Process Hierarchy API client (OData v4) - CALM_PH.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ApiError;
use crate::odata::{ODataClient, ODataCollection, ODataQuery};

/// Hierarchy Node entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HierarchyNode {
    pub uuid: Option<String>,
    pub display_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub hierarchy_level: Option<i32>,
    pub sequence: Option<i32>,
    pub parent_titles: Option<String>,
    pub parent_node_uuid: Option<String>,
    pub root_node_uuid: Option<String>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
}

/// Request to create a hierarchy node.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHierarchyNodeRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_node_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<i32>,
}

/// Request to update a hierarchy node.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHierarchyNodeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<i32>,
}

/// Process Hierarchy API client.
#[derive(Clone)]
pub struct ProcessHierarchyClient {
    odata_client: ODataClient,
}

impl ProcessHierarchyClient {
    pub fn new(odata_client: ODataClient) -> Self {
        Self { odata_client }
    }

    pub async fn list_nodes(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<ODataCollection<HierarchyNode>, ApiError> {
        self.odata_client
            .get_collection("/HierarchyNodes", query)
            .await
    }

    pub async fn get_node(&self, uuid: &str) -> Result<HierarchyNode, ApiError> {
        self.odata_client
            .get_entity_by_uuid("/HierarchyNodes", uuid)
            .await
    }

    pub async fn get_node_with_expand(
        &self,
        uuid: &str,
        expand: &[&str],
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_entity_with_expand("/HierarchyNodes", uuid, expand)
            .await
    }

    pub async fn create_node(
        &self,
        request: &CreateHierarchyNodeRequest,
    ) -> Result<HierarchyNode, ApiError> {
        self.odata_client
            .create_entity("/HierarchyNodes", request)
            .await
    }

    pub async fn update_node(
        &self,
        uuid: &str,
        request: &UpdateHierarchyNodeRequest,
    ) -> Result<HierarchyNode, ApiError> {
        self.odata_client
            .update_entity_by_uuid("/HierarchyNodes", uuid, request)
            .await
    }

    pub async fn delete_node(&self, uuid: &str) -> Result<(), ApiError> {
        self.odata_client
            .delete_entity_by_uuid("/HierarchyNodes", uuid)
            .await
    }
}

impl std::fmt::Debug for ProcessHierarchyClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessHierarchyClient").finish()
    }
}
