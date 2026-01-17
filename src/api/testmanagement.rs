//! Test Management API client (OData v4) - CALM_TM.

use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::odata::{ODataClient, ODataCollection, ODataQuery};

/// Manual Test Case entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status_code: Option<String>,
    pub project_id: Option<String>,
    pub modified_at: Option<String>,
    pub created_at: Option<String>,
}

/// Test Activity entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestActivity {
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub sequence: Option<i32>,
    #[serde(rename = "parent_ID")]
    pub parent_id: Option<String>,
    pub modified_at: Option<String>,
}

/// Test Action entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestAction {
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub expected_result: Option<String>,
    pub sequence: Option<i32>,
    pub is_evidence_required: Option<bool>,
    #[serde(rename = "parent_ID")]
    pub parent_id: Option<String>,
    pub modified_at: Option<String>,
}

/// Request to create a test case.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTestCaseRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

/// Request to update a test case.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTestCaseRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
}

/// Request to create a test activity.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTestActivityRequest {
    pub title: String,
    #[serde(rename = "parent_ID")]
    pub parent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<i32>,
}

/// Request to create a test action.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTestActionRequest {
    pub title: String,
    #[serde(rename = "parent_ID")]
    pub parent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_evidence_required: Option<bool>,
}

/// Test Management API client.
#[derive(Clone)]
pub struct TestManagementClient {
    odata_client: ODataClient,
}

impl TestManagementClient {
    pub fn new(odata_client: ODataClient) -> Self {
        Self { odata_client }
    }

    pub async fn list_testcases(&self, query: Option<ODataQuery>) -> Result<ODataCollection<TestCase>, ApiError> {
        self.odata_client.get_collection("/ManualTestCases", query).await
    }

    pub async fn get_testcase(&self, uuid: &str) -> Result<TestCase, ApiError> {
        self.odata_client.get_entity_by_uuid("/ManualTestCases", uuid).await
    }

    pub async fn create_testcase(&self, request: &CreateTestCaseRequest) -> Result<TestCase, ApiError> {
        self.odata_client.create_entity("/ManualTestCases", request).await
    }

    pub async fn update_testcase(&self, uuid: &str, request: &UpdateTestCaseRequest) -> Result<TestCase, ApiError> {
        self.odata_client.update_entity_by_uuid("/ManualTestCases", uuid, request).await
    }

    pub async fn delete_testcase(&self, uuid: &str) -> Result<(), ApiError> {
        self.odata_client.delete_entity_by_uuid("/ManualTestCases", uuid).await
    }

    pub async fn list_activities(&self, query: Option<ODataQuery>) -> Result<ODataCollection<TestActivity>, ApiError> {
        self.odata_client.get_collection("/Activities", query).await
    }

    pub async fn create_activity(&self, request: &CreateTestActivityRequest) -> Result<TestActivity, ApiError> {
        self.odata_client.create_entity("/Activities", request).await
    }

    pub async fn list_actions(&self, query: Option<ODataQuery>) -> Result<ODataCollection<TestAction>, ApiError> {
        self.odata_client.get_collection("/Actions", query).await
    }

    pub async fn create_action(&self, request: &CreateTestActionRequest) -> Result<TestAction, ApiError> {
        self.odata_client.create_entity("/Actions", request).await
    }
}

impl std::fmt::Debug for TestManagementClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestManagementClient").finish()
    }
}
