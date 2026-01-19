//! Documents API client (OData v4) - CALM_SD.

use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::odata::{ODataClient, ODataCollection, ODataQuery};

/// Document entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub uuid: Option<String>,
    #[serde(rename = "displayId")]
    pub display_id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub status_code: Option<i32>,
    pub priority_code: Option<i32>,
    #[serde(rename = "documentTypeCode")]
    pub type_code: Option<String>,
    pub source_code: Option<String>,
    pub project_id: Option<String>,
    pub scope_id: Option<String>,
    pub modified_at: Option<String>,
    pub created_at: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Document type code.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentType {
    pub code: String,
    pub name: String,
}

/// Document status code.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentStatus {
    pub code: String,
    pub name: String,
}

/// Request to create a document.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDocumentRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_code: Option<String>,
}

/// Request to update a document.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_code: Option<String>,
}

/// Documents API client.
#[derive(Clone)]
pub struct DocumentsClient {
    odata_client: ODataClient,
}

impl DocumentsClient {
    /// Create a new Documents client.
    pub fn new(odata_client: ODataClient) -> Self {
        Self { odata_client }
    }

    /// List documents with optional OData query.
    pub async fn list_documents(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<ODataCollection<Document>, ApiError> {
        self.odata_client
            .get_collection("/Documents", query)
            .await
    }

    /// Get a single document by UUID.
    pub async fn get_document(&self, uuid: &str) -> Result<Document, ApiError> {
        self.odata_client
            .get_entity_by_uuid("/Documents", uuid)
            .await
    }

    /// Create a new document.
    pub async fn create_document(
        &self,
        request: &CreateDocumentRequest,
    ) -> Result<Document, ApiError> {
        self.odata_client
            .create_entity("/Documents", request)
            .await
    }

    /// Update an existing document.
    pub async fn update_document(
        &self,
        uuid: &str,
        request: &UpdateDocumentRequest,
    ) -> Result<Document, ApiError> {
        self.odata_client
            .update_entity_by_uuid("/Documents", uuid, request)
            .await
    }

    /// Delete a document.
    pub async fn delete_document(&self, uuid: &str) -> Result<(), ApiError> {
        self.odata_client
            .delete_entity_by_uuid("/Documents", uuid)
            .await
    }

    /// List document types.
    pub async fn list_types(&self) -> Result<ODataCollection<DocumentType>, ApiError> {
        self.odata_client
            .get_collection("/DocumentTypes", None)
            .await
    }

    /// List document statuses.
    pub async fn list_statuses(&self) -> Result<ODataCollection<DocumentStatus>, ApiError> {
        self.odata_client
            .get_collection("/DocumentStatuses", None)
            .await
    }
}

impl std::fmt::Debug for DocumentsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentsClient").finish()
    }
}
