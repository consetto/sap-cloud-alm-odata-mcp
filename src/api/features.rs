//! Features API client (OData v4) - CALM_CDM_ODATA.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ApiError;
use crate::odata::{ODataClient, ODataCollection, ODataQuery};

/// Feature entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub uuid: Option<String>,
    pub display_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub project_id: Option<String>,
    pub status_code: Option<String>,
    pub priority_code: Option<i32>,
    pub release_id: Option<String>,
    pub scope_id: Option<String>,
    pub responsible_id: Option<String>,
    pub modified_at: Option<String>,
    pub created_at: Option<String>,
    #[serde(rename = "type")]
    pub feature_type: Option<String>,
    pub workstream_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// External reference entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalReference {
    pub id: Option<String>,
    pub parent_uuid: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

/// Priority code entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriorityCode {
    pub code: String,
    pub name: String,
}

/// Status code entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusCode {
    pub code: String,
    pub name: String,
}

/// Request to create a feature.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeatureRequest {
    pub title: String,
    pub project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
}

/// Request to update a feature.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFeatureRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
}

/// Request to create an external reference.
#[derive(Debug, Clone, Serialize)]
pub struct CreateExternalReferenceRequest {
    pub id: String,
    pub parent_uuid: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Features API client.
#[derive(Clone)]
pub struct FeaturesClient {
    odata_client: ODataClient,
}

impl FeaturesClient {
    /// Create a new Features client.
    pub fn new(odata_client: ODataClient) -> Self {
        Self { odata_client }
    }

    /// List features with optional OData query.
    pub async fn list_features(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<ODataCollection<Feature>, ApiError> {
        self.odata_client.get_collection("/Features", query).await
    }

    /// Get a single feature by UUID.
    pub async fn get_feature(&self, uuid: &str) -> Result<Feature, ApiError> {
        self.odata_client
            .get_entity_by_uuid("/Features", uuid)
            .await
    }

    /// Get a single feature by display ID (e.g., "6-123").
    ///
    /// # Arguments
    ///
    /// * `display_id` - The display ID of the feature (format: "projectNumber-featureNumber")
    ///
    /// # Returns
    ///
    /// The feature matching the display ID, or an error if not found.
    ///
    /// # Errors
    ///
    /// Returns `ApiError::HttpError` with NOT_FOUND status if no feature matches the display ID.
    pub async fn get_feature_by_display_id(&self, display_id: &str) -> Result<Feature, ApiError> {
        let query = ODataQuery::new()
            .filter(format!("displayId eq '{}'", display_id))
            .top(1);
        let collection = self.list_features(Some(query)).await?;
        collection.value.into_iter().next().ok_or_else(|| {
            ApiError::HttpError {
                status: reqwest::StatusCode::NOT_FOUND,
                body: format!("Feature with displayId '{}' not found", display_id),
            }
        })
    }

    /// Get a feature by display ID with expanded relations.
    ///
    /// # Arguments
    ///
    /// * `display_id` - The display ID of the feature (format: "projectNumber-featureNumber")
    /// * `expand` - List of navigation properties to expand
    ///
    /// # Returns
    ///
    /// The feature as a JSON Value with expanded relations, or an error if not found.
    pub async fn get_feature_by_display_id_with_expand(
        &self,
        display_id: &str,
        expand: &[&str],
    ) -> Result<serde_json::Value, ApiError> {
        // First, find the feature by display ID to get its UUID
        let feature = self.get_feature_by_display_id(display_id).await?;
        let uuid = feature.uuid.ok_or_else(|| ApiError::HttpError {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            body: "Feature UUID is missing".to_string(),
        })?;
        // Then fetch with expanded relations using the UUID
        self.get_feature_with_expand(&uuid, expand).await
    }

    /// Get a feature with expanded relations.
    pub async fn get_feature_with_expand(
        &self,
        uuid: &str,
        expand: &[&str],
    ) -> Result<Value, ApiError> {
        self.odata_client
            .get_entity_with_expand("/Features", uuid, expand)
            .await
    }

    /// Create a new feature.
    pub async fn create_feature(
        &self,
        request: &CreateFeatureRequest,
    ) -> Result<Feature, ApiError> {
        self.odata_client.create_entity("/Features", request).await
    }

    /// Update an existing feature.
    pub async fn update_feature(
        &self,
        uuid: &str,
        request: &UpdateFeatureRequest,
    ) -> Result<Feature, ApiError> {
        self.odata_client
            .update_entity_by_uuid("/Features", uuid, request)
            .await
    }

    /// Delete a feature.
    pub async fn delete_feature(&self, uuid: &str) -> Result<(), ApiError> {
        self.odata_client
            .delete_entity_by_uuid("/Features", uuid)
            .await
    }

    /// List external references with optional query.
    pub async fn list_external_references(
        &self,
        query: Option<ODataQuery>,
    ) -> Result<ODataCollection<ExternalReference>, ApiError> {
        self.odata_client
            .get_collection("/ExternalReferences", query)
            .await
    }

    /// Create an external reference.
    pub async fn create_external_reference(
        &self,
        request: &CreateExternalReferenceRequest,
    ) -> Result<ExternalReference, ApiError> {
        self.odata_client
            .create_entity("/ExternalReferences", request)
            .await
    }

    /// Delete an external reference.
    pub async fn delete_external_reference(
        &self,
        id: &str,
        parent_uuid: &str,
    ) -> Result<(), ApiError> {
        let endpoint = format!("/ExternalReferences/{}/{}", id, parent_uuid);
        self.odata_client.delete_entity_by_uuid(&endpoint, "").await
    }

    /// List priority codes.
    pub async fn list_priorities(&self) -> Result<ODataCollection<PriorityCode>, ApiError> {
        self.odata_client
            .get_collection("/FeaturePriorities", None)
            .await
    }

    /// List status codes.
    pub async fn list_statuses(&self) -> Result<ODataCollection<StatusCode>, ApiError> {
        self.odata_client
            .get_collection("/FeatureStatus", None)
            .await
    }
}

impl std::fmt::Debug for FeaturesClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeaturesClient").finish()
    }
}
