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
    /// Creates a new Process Hierarchy API client.
    ///
    /// # Arguments
    ///
    /// * `odata_client` - The OData client configured for the Process Hierarchy API endpoint
    pub fn new(odata_client: ODataClient) -> Self {
        Self { odata_client }
    }

    /// Lists hierarchy nodes with optional OData query parameters.
    ///
    /// Hierarchy nodes represent the process structure in SAP Cloud ALM,
    /// organized in a tree with parent-child relationships.
    ///
    /// # Arguments
    ///
    /// * `query` - Optional OData query for filtering, sorting, and pagination
    ///
    /// # Returns
    ///
    /// A collection of hierarchy nodes matching the query criteria.
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if the request fails or response parsing fails.
    pub async fn list_nodes(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<ODataCollection<HierarchyNode>, ApiError> {
        self.odata_client
            .get_collection("/HierarchyNodes", query)
            .await
    }

    /// Retrieves a single hierarchy node by its UUID.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The unique identifier of the hierarchy node
    ///
    /// # Returns
    ///
    /// The hierarchy node with the specified UUID.
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if the node is not found or request fails.
    pub async fn get_node(&self, uuid: &str) -> Result<HierarchyNode, ApiError> {
        self.odata_client
            .get_entity_by_uuid("/HierarchyNodes", uuid)
            .await
    }

    /// Retrieves a hierarchy node with expanded navigation properties.
    ///
    /// Use this to fetch related entities (parent node, child nodes, external references)
    /// in a single request.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The unique identifier of the hierarchy node
    /// * `expand` - Navigation properties to expand (e.g., `["toParentNode", "toChildNodes"]`)
    ///
    /// # Returns
    ///
    /// The hierarchy node as raw JSON with expanded relations included.
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if the node is not found or request fails.
    pub async fn get_node_with_expand(
        &self,
        uuid: &str,
        expand: &[&str],
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_entity_with_expand("/HierarchyNodes", uuid, expand)
            .await
    }

    /// Creates a new hierarchy node.
    ///
    /// # Arguments
    ///
    /// * `request` - The node creation request containing title and optional parent/sequence
    ///
    /// # Returns
    ///
    /// The newly created hierarchy node with server-generated fields populated.
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if creation fails due to validation or server errors.
    pub async fn create_node(
        &self,
        request: &CreateHierarchyNodeRequest,
    ) -> Result<HierarchyNode, ApiError> {
        self.odata_client
            .create_entity("/HierarchyNodes", request)
            .await
    }

    /// Updates an existing hierarchy node.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The unique identifier of the hierarchy node to update
    /// * `request` - The update request containing fields to modify
    ///
    /// # Returns
    ///
    /// The updated hierarchy node with new values applied.
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if the node is not found or update fails.
    pub async fn update_node(
        &self,
        uuid: &str,
        request: &UpdateHierarchyNodeRequest,
    ) -> Result<HierarchyNode, ApiError> {
        self.odata_client
            .update_entity_by_uuid("/HierarchyNodes", uuid, request)
            .await
    }

    /// Deletes a hierarchy node by its UUID.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The unique identifier of the hierarchy node to delete
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if the node is not found or deletion fails.
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
