//! MCP Server implementation with SAP Cloud ALM tools.

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, ErrorCode, ErrorData as McpError, Implementation,
        ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::{
    AnalyticsClient, DocumentsClient, FeaturesClient, LogsClient, ProcessHierarchyClient,
    ProcessMonitoringClient, ProjectsClient, TasksClient, TestManagementClient,
};
use crate::api::documents::{CreateDocumentRequest, UpdateDocumentRequest};
use crate::api::features::{CreateExternalReferenceRequest, CreateFeatureRequest, UpdateFeatureRequest};
use crate::api::logs::{GetLogsParams, PostLogsParams};
use crate::api::processhierarchy::{CreateHierarchyNodeRequest, UpdateHierarchyNodeRequest};
use crate::api::projects::CreateProjectRequest;
use crate::api::tasks::{CreateTaskCommentRequest, CreateTaskRequest, ListTasksParams, UpdateTaskRequest};
use crate::api::testmanagement::{
    CreateTestActionRequest, CreateTestActivityRequest, CreateTestCaseRequest, UpdateTestCaseRequest,
};
use crate::debug::DebugLogger;
use crate::odata::ODataQuery;

/// SAP Cloud ALM MCP Server.
#[derive(Clone)]
pub struct SapCloudAlmServer {
    features_client: FeaturesClient,
    documents_client: DocumentsClient,
    tasks_client: TasksClient,
    projects_client: ProjectsClient,
    testmanagement_client: TestManagementClient,
    processhierarchy_client: ProcessHierarchyClient,
    analytics_client: AnalyticsClient,
    processmonitoring_client: ProcessMonitoringClient,
    logs_client: LogsClient,
    debug: Arc<DebugLogger>,
    tool_router: ToolRouter<Self>,
}

impl SapCloudAlmServer {
    pub fn new(
        features_client: FeaturesClient,
        documents_client: DocumentsClient,
        tasks_client: TasksClient,
        projects_client: ProjectsClient,
        testmanagement_client: TestManagementClient,
        processhierarchy_client: ProcessHierarchyClient,
        analytics_client: AnalyticsClient,
        processmonitoring_client: ProcessMonitoringClient,
        logs_client: LogsClient,
        debug: Arc<DebugLogger>,
    ) -> Self {
        Self {
            features_client,
            documents_client,
            tasks_client,
            projects_client,
            testmanagement_client,
            processhierarchy_client,
            analytics_client,
            processmonitoring_client,
            logs_client,
            debug,
            tool_router: Self::tool_router(),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert any error to McpError
fn to_mcp_error<E: std::fmt::Display>(e: E) -> McpError {
    McpError {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::from(e.to_string()),
        data: None,
    }
}

// ============================================================================
// Tool Parameter Structs
// ============================================================================

fn build_odata_query(
    filter: Option<String>,
    select: Option<String>,
    expand: Option<String>,
    orderby: Option<String>,
    top: Option<u32>,
    skip: Option<u32>,
) -> Option<ODataQuery> {
    if filter.is_none()
        && select.is_none()
        && expand.is_none()
        && orderby.is_none()
        && top.is_none()
        && skip.is_none()
    {
        return None;
    }

    let mut query = ODataQuery::new();
    if let Some(f) = filter {
        query = query.filter(&f);
    }
    if let Some(s) = select {
        query = query.select(s.split(',').map(|x| x.trim().to_string()).collect());
    }
    if let Some(e) = expand {
        query = query.expand(e.split(',').map(|x| x.trim().to_string()).collect());
    }
    if let Some(o) = orderby {
        // Parse orderby as "field asc" or "field desc" or just "field"
        let parts: Vec<&str> = o.split_whitespace().collect();
        let field = parts.first().map(|s| s.to_string()).unwrap_or_default();
        let order = if parts.get(1).map(|s| s.to_lowercase()).as_deref() == Some("desc") {
            crate::odata::SortOrder::Desc
        } else {
            crate::odata::SortOrder::Asc
        };
        query = query.orderby(&field, order);
    }
    if let Some(t) = top {
        query = query.top(t);
    }
    if let Some(s) = skip {
        query = query.skip(s);
    }
    Some(query)
}

// Feature tools params
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListFeaturesParams {
    /// OData $filter expression (e.g., "projectId eq 'abc'")
    pub filter: Option<String>,
    /// Comma-separated list of fields to select
    pub select: Option<String>,
    /// Comma-separated list of navigation properties to expand
    pub expand: Option<String>,
    /// OData $orderby expression (e.g., "createdAt desc")
    pub orderby: Option<String>,
    /// Maximum number of records to return
    pub top: Option<u32>,
    /// Number of records to skip for pagination
    pub skip: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetFeatureParams {
    /// Feature UUID
    pub uuid: String,
    /// Navigation properties to expand (comma-separated): toProject, toRelease, toScope, toStatus, toPriority, toTransports, toExternalReferences
    pub expand: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateFeatureParams {
    /// Feature title (required)
    pub title: String,
    /// Project ID (required)
    pub project_id: String,
    /// Feature description
    pub description: Option<String>,
    /// Status code
    pub status_code: Option<String>,
    /// Priority code
    pub priority_code: Option<String>,
    /// Release ID
    pub release_id: Option<String>,
    /// Scope ID
    pub scope_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateFeatureParams {
    /// Feature UUID
    pub uuid: String,
    /// New title
    pub title: Option<String>,
    /// New description
    pub description: Option<String>,
    /// New status code
    pub status_code: Option<String>,
    /// New priority code
    pub priority_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UuidParams {
    /// UUID
    pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct IdParams {
    /// ID
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListExternalReferencesParams {
    /// OData $filter expression
    pub filter: Option<String>,
    /// Comma-separated list of fields to select
    pub select: Option<String>,
    /// Maximum number of records to return
    pub top: Option<u32>,
    /// Number of records to skip for pagination
    pub skip: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateExternalReferenceParams {
    /// Parent feature UUID
    pub parent_uuid: String,
    /// External reference ID
    pub id: String,
    /// Reference name
    pub name: String,
    /// Reference URL
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteExternalReferenceParams {
    /// External reference ID
    pub id: String,
    /// Parent feature UUID
    pub parent_uuid: String,
}

// Document tools params
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListDocumentsParams {
    /// OData $filter expression
    pub filter: Option<String>,
    /// Comma-separated list of fields to select
    pub select: Option<String>,
    /// OData $orderby expression
    pub orderby: Option<String>,
    /// Maximum number of records to return
    pub top: Option<u32>,
    /// Number of records to skip for pagination
    pub skip: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDocumentParams {
    /// Document title (required)
    pub title: String,
    /// HTML content
    pub content: Option<String>,
    /// Project ID
    pub project_id: Option<String>,
    /// Document type code
    pub type_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateDocumentParams {
    /// Document UUID
    pub uuid: String,
    /// New title
    pub title: Option<String>,
    /// New HTML content
    pub content: Option<String>,
    /// New status code
    pub status_code: Option<String>,
}

// Task tools params
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListTasksToolParams {
    /// Project ID (required)
    pub project_id: String,
    /// Task type filter
    pub task_type: Option<String>,
    /// Status filter
    pub status: Option<String>,
    /// Sub-status filter
    pub sub_status: Option<String>,
    /// Assignee ID filter
    pub assignee_id: Option<String>,
    /// Tags filter (comma-separated)
    pub tags: Option<String>,
    /// Number of records to skip
    pub offset: Option<u32>,
    /// Maximum number of records to return
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateTaskParams {
    /// Project ID (required)
    pub project_id: String,
    /// Task title (required)
    pub title: String,
    /// Task type (required)
    pub task_type: String,
    /// Task description
    pub description: Option<String>,
    /// Assignee ID
    pub assignee_id: Option<String>,
    /// Due date (ISO format)
    pub due_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateTaskParams {
    /// Task UUID
    pub uuid: String,
    /// New title
    pub title: Option<String>,
    /// New description
    pub description: Option<String>,
    /// New status
    pub status: Option<String>,
    /// New assignee ID
    pub assignee_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TaskIdParams {
    /// Task UUID
    pub task_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateTaskCommentParams {
    /// Task UUID
    pub task_id: String,
    /// Comment content
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProjectIdParams {
    /// Project ID
    pub project_id: String,
}

// Project tools params
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateProjectParams {
    /// Project name (required)
    pub name: String,
    /// Program ID
    pub program_id: Option<String>,
}

// Test Management tools params
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ODataListParams {
    /// OData $filter expression
    pub filter: Option<String>,
    /// Comma-separated list of fields to select
    pub select: Option<String>,
    /// Comma-separated list of navigation properties to expand
    pub expand: Option<String>,
    /// OData $orderby expression
    pub orderby: Option<String>,
    /// Maximum number of records to return
    pub top: Option<u32>,
    /// Number of records to skip for pagination
    pub skip: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateTestcaseParams {
    /// Test case title (required)
    pub title: String,
    /// Test case description
    pub description: Option<String>,
    /// Project ID
    pub project_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateTestcaseParams {
    /// Test case UUID
    pub uuid: String,
    /// New title
    pub title: Option<String>,
    /// New description
    pub description: Option<String>,
    /// New status code
    pub status_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateTestActivityParams {
    /// Activity title (required)
    pub title: String,
    /// Parent test case UUID (required)
    pub parent_id: String,
    /// Activity description
    pub description: Option<String>,
    /// Sequence number
    pub sequence: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateTestActionParams {
    /// Action title (required)
    pub title: String,
    /// Parent activity UUID (required)
    pub parent_id: String,
    /// Action description
    pub description: Option<String>,
    /// Expected result
    pub expected_result: Option<String>,
    /// Sequence number
    pub sequence: Option<i32>,
    /// Whether evidence is required
    pub is_evidence_required: Option<bool>,
}

// Process Hierarchy tools params
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetHierarchyNodeParams {
    /// Node UUID
    pub uuid: String,
    /// Navigation properties to expand (comma-separated): toParentNode, toChildNodes, toExternalReferences
    pub expand: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateHierarchyNodeParams {
    /// Node title (required)
    pub title: String,
    /// Parent node UUID
    pub parent_node_uuid: Option<String>,
    /// Node description
    pub description: Option<String>,
    /// Sequence number
    pub sequence: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateHierarchyNodeParams {
    /// Node UUID
    pub uuid: String,
    /// New title
    pub title: Option<String>,
    /// New description
    pub description: Option<String>,
    /// New sequence
    pub sequence: Option<i32>,
}

// Analytics tools params
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryDatasetParams {
    /// Data provider name (required)
    pub provider: String,
    /// OData $filter expression
    pub filter: Option<String>,
    /// Comma-separated list of fields to select
    pub select: Option<String>,
    /// OData $orderby expression
    pub orderby: Option<String>,
    /// Maximum number of records to return
    pub top: Option<u32>,
    /// Number of records to skip for pagination
    pub skip: Option<u32>,
}

// Logs tools params
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetLogsToolParams {
    /// Provider name (required)
    pub provider: String,
    /// Log format
    pub format: Option<String>,
    /// API version
    pub version: Option<String>,
    /// Time period (e.g., "1h", "24h")
    pub period: Option<String>,
    /// Start timestamp (ISO format)
    pub from: Option<String>,
    /// End timestamp (ISO format)
    pub to: Option<String>,
    /// Maximum number of logs
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Service ID filter
    pub service_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PostLogsToolParams {
    /// Use case identifier (required)
    pub use_case: String,
    /// Service ID (required)
    pub service_id: String,
    /// API version
    pub version: Option<String>,
    /// Development mode flag
    pub dev: Option<bool>,
    /// Tag for the logs
    pub tag: Option<String>,
    /// Log data (JSON array of log entries)
    pub logs: Value,
}

// ============================================================================
// Tool Implementations
// ============================================================================

#[tool_router]
impl SapCloudAlmServer {
    // ========================================================================
    // Features API Tools
    // ========================================================================

    #[tool(description = "List features from SAP Cloud ALM with OData filtering. Supports $filter, $select, $expand, $orderby, $top, $skip.")]
    async fn list_features(&self, Parameters(params): Parameters<ListFeaturesParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_features", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.features_client.list_features(query).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_features", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Get a single feature by UUID. Optionally expand related entities.")]
    async fn get_feature(&self, Parameters(params): Parameters<GetFeatureParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_feature", &json!({"uuid": params.uuid, "expand": params.expand}));

        let result = if let Some(ref expand) = params.expand {
            let expand_list: Vec<&str> = expand.split(',').map(|s: &str| s.trim()).collect();
            self.features_client.get_feature_with_expand(&params.uuid, &expand_list).await
        } else {
            self.features_client.get_feature(&params.uuid).await
                .map(|f| serde_json::to_value(f).unwrap())
        };

        let json = result.map_err(to_mcp_error)?;
        self.debug.log_tool_result("get_feature", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Create a new feature. Required: title and project_id.")]
    async fn create_feature(&self, Parameters(params): Parameters<CreateFeatureParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_feature", &json!(params));

        let request = CreateFeatureRequest {
            title: params.title,
            project_id: params.project_id,
            description: params.description,
            status_code: params.status_code,
            priority_code: params.priority_code,
            release_id: params.release_id,
            scope_id: params.scope_id,
        };

        let result = self.features_client.create_feature(&request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_feature", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Update an existing feature. Only provided fields will be updated.")]
    async fn update_feature(&self, Parameters(params): Parameters<UpdateFeatureParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("update_feature", &json!(params));

        let request = UpdateFeatureRequest {
            title: params.title,
            description: params.description,
            status_code: params.status_code,
            priority_code: params.priority_code,
            release_id: None,
            scope_id: None,
        };

        let result = self.features_client.update_feature(&params.uuid, &request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("update_feature", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Delete a feature by UUID.")]
    async fn delete_feature(&self, Parameters(params): Parameters<UuidParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("delete_feature", &json!({"uuid": params.uuid}));

        self.features_client.delete_feature(&params.uuid).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("delete_feature", &json!({"deleted": true}));

        Ok(CallToolResult::success(vec![Content::text(json!({"deleted": true, "uuid": params.uuid}).to_string())]))
    }

    #[tool(description = "List external references with OData filtering.")]
    async fn list_external_references(&self, Parameters(params): Parameters<ListExternalReferencesParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_external_references", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            None,
            None,
            params.top,
            params.skip,
        );

        let result = self.features_client.list_external_references(query).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_external_references", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Create an external reference for a feature.")]
    async fn create_external_reference(&self, Parameters(params): Parameters<CreateExternalReferenceParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_external_reference", &json!(params));

        let request = CreateExternalReferenceRequest {
            parent_uuid: params.parent_uuid,
            id: params.id,
            name: params.name,
            url: Some(params.url),
        };

        let result = self.features_client.create_external_reference(&request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_external_reference", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Delete an external reference.")]
    async fn delete_external_reference(&self, Parameters(params): Parameters<DeleteExternalReferenceParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("delete_external_reference", &json!(params));

        self.features_client.delete_external_reference(&params.id, &params.parent_uuid).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("delete_external_reference", &json!({"deleted": true}));

        Ok(CallToolResult::success(vec![Content::text(json!({"deleted": true}).to_string())]))
    }

    #[tool(description = "List available feature priorities.")]
    async fn list_feature_priorities(&self) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_feature_priorities", &json!({}));

        let result = self.features_client.list_priorities().await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_feature_priorities", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "List available feature statuses.")]
    async fn list_feature_statuses(&self) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_feature_statuses", &json!({}));

        let result = self.features_client.list_statuses().await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_feature_statuses", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    // ========================================================================
    // Documents API Tools
    // ========================================================================

    #[tool(description = "List documents from SAP Cloud ALM with OData filtering.")]
    async fn list_documents(&self, Parameters(params): Parameters<ListDocumentsParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_documents", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            None,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.documents_client.list_documents(query).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_documents", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Get a single document by UUID.")]
    async fn get_document(&self, Parameters(params): Parameters<UuidParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_document", &json!({"uuid": params.uuid}));

        let result = self.documents_client.get_document(&params.uuid).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("get_document", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Create a new document. Required: title.")]
    async fn create_document(&self, Parameters(params): Parameters<CreateDocumentParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_document", &json!(params));

        let request = CreateDocumentRequest {
            title: params.title,
            content: params.content,
            project_id: params.project_id,
            type_code: params.type_code,
            status_code: None,
            priority_code: None,
        };

        let result = self.documents_client.create_document(&request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_document", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Update an existing document.")]
    async fn update_document(&self, Parameters(params): Parameters<UpdateDocumentParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("update_document", &json!(params));

        let request = UpdateDocumentRequest {
            title: params.title,
            content: params.content,
            status_code: params.status_code,
            priority_code: None,
            type_code: None,
        };

        let result = self.documents_client.update_document(&params.uuid, &request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("update_document", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Delete a document by UUID.")]
    async fn delete_document(&self, Parameters(params): Parameters<UuidParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("delete_document", &json!({"uuid": params.uuid}));

        self.documents_client.delete_document(&params.uuid).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("delete_document", &json!({"deleted": true}));

        Ok(CallToolResult::success(vec![Content::text(json!({"deleted": true, "uuid": params.uuid}).to_string())]))
    }

    #[tool(description = "List available document types.")]
    async fn list_document_types(&self) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_document_types", &json!({}));

        let result = self.documents_client.list_types().await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_document_types", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "List available document statuses.")]
    async fn list_document_statuses(&self) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_document_statuses", &json!({}));

        let result = self.documents_client.list_statuses().await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_document_statuses", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    // ========================================================================
    // Tasks API Tools
    // ========================================================================

    #[tool(description = "List tasks for a project. Required: project_id. Supports filtering by type, status, assignee, tags.")]
    async fn list_tasks(&self, Parameters(params): Parameters<ListTasksToolParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_tasks", &json!(params));

        // Convert comma-separated tags to Vec if provided
        let tags: Option<Vec<String>> = params.tags.map(|t: String| t.split(',').map(|s: &str| s.trim().to_string()).collect());

        let list_params = ListTasksParams {
            project_id: params.project_id,
            task_type: params.task_type,
            status: params.status,
            sub_status: params.sub_status,
            assignee_id: params.assignee_id,
            tags,
            offset: params.offset,
            limit: params.limit,
            ..Default::default()
        };

        let result = self.tasks_client.list_tasks(&list_params).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_tasks", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Get a single task by UUID with full details.")]
    async fn get_task(&self, Parameters(params): Parameters<UuidParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_task", &json!({"uuid": params.uuid}));

        let result = self.tasks_client.get_task(&params.uuid).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("get_task", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Create a new task. Required: project_id, title, task_type.")]
    async fn create_task(&self, Parameters(params): Parameters<CreateTaskParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_task", &json!(params));

        let request = CreateTaskRequest {
            project_id: params.project_id,
            title: params.title,
            task_type: params.task_type,
            description: params.description,
            priority_id: None,
            assignee_id: params.assignee_id,
            due_date: params.due_date,
        };

        let result = self.tasks_client.create_task(&request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_task", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Update an existing task.")]
    async fn update_task(&self, Parameters(params): Parameters<UpdateTaskParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("update_task", &json!(params));

        let request = UpdateTaskRequest {
            title: params.title,
            description: params.description,
            status: params.status,
            priority_id: None,
            assignee_id: params.assignee_id,
            due_date: None,
        };

        let result = self.tasks_client.update_task(&params.uuid, &request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("update_task", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Delete a task by UUID.")]
    async fn delete_task(&self, Parameters(params): Parameters<UuidParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("delete_task", &json!({"uuid": params.uuid}));

        self.tasks_client.delete_task(&params.uuid).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("delete_task", &json!({"deleted": true}));

        Ok(CallToolResult::success(vec![Content::text(json!({"deleted": true, "uuid": params.uuid}).to_string())]))
    }

    #[tool(description = "List comments on a task.")]
    async fn list_task_comments(&self, Parameters(params): Parameters<TaskIdParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_task_comments", &json!({"task_id": params.task_id}));

        let result = self.tasks_client.list_task_comments(&params.task_id).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_task_comments", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Add a comment to a task.")]
    async fn create_task_comment(&self, Parameters(params): Parameters<CreateTaskCommentParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_task_comment", &json!(params));

        let request = CreateTaskCommentRequest {
            content: params.content,
        };

        let result = self.tasks_client.create_task_comment(&params.task_id, &request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_task_comment", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "List external references for a task.")]
    async fn list_task_references(&self, Parameters(params): Parameters<TaskIdParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_task_references", &json!({"task_id": params.task_id}));

        let result = self.tasks_client.list_task_references(&params.task_id).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_task_references", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "List workstreams for a project.")]
    async fn list_workstreams(&self, Parameters(params): Parameters<ProjectIdParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_workstreams", &json!({"project_id": params.project_id}));

        let result = self.tasks_client.list_workstreams(&params.project_id).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_workstreams", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "List deliverables for a project.")]
    async fn list_deliverables(&self, Parameters(params): Parameters<ProjectIdParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_deliverables", &json!({"project_id": params.project_id}));

        let result = self.tasks_client.list_deliverables(&params.project_id).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_deliverables", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    // ========================================================================
    // Projects API Tools
    // ========================================================================

    #[tool(description = "List all accessible projects.")]
    async fn list_projects(&self) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_projects", &json!({}));

        let result = self.projects_client.list_projects().await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_projects", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Get project details by ID.")]
    async fn get_project(&self, Parameters(params): Parameters<IdParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_project", &json!({"id": params.id}));

        let result = self.projects_client.get_project(&params.id).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("get_project", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Create a new project.")]
    async fn create_project(&self, Parameters(params): Parameters<CreateProjectParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_project", &json!(params));

        let request = CreateProjectRequest {
            name: params.name,
            description: None,
            program_id: params.program_id,
        };

        let result = self.projects_client.create_project(&request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_project", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "List timeboxes (sprints) for a project.")]
    async fn list_project_timeboxes(&self, Parameters(params): Parameters<ProjectIdParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_project_timeboxes", &json!({"project_id": params.project_id}));

        let result = self.projects_client.list_timeboxes(&params.project_id).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_project_timeboxes", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "List team members for a project.")]
    async fn list_project_teams(&self, Parameters(params): Parameters<ProjectIdParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_project_teams", &json!({"project_id": params.project_id}));

        let result = self.projects_client.list_team_members(&params.project_id).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_project_teams", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "List all programs.")]
    async fn list_programs(&self) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_programs", &json!({}));

        let result = self.projects_client.list_programs().await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_programs", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Get program details by ID.")]
    async fn get_program(&self, Parameters(params): Parameters<IdParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_program", &json!({"id": params.id}));

        let result = self.projects_client.get_program(&params.id).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("get_program", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    // ========================================================================
    // Test Management API Tools
    // ========================================================================

    #[tool(description = "List manual test cases with OData filtering.")]
    async fn list_testcases(&self, Parameters(params): Parameters<ODataListParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_testcases", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.testmanagement_client.list_testcases(query).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_testcases", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Get a test case by UUID.")]
    async fn get_testcase(&self, Parameters(params): Parameters<UuidParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_testcase", &json!({"uuid": params.uuid}));

        let result = self.testmanagement_client.get_testcase(&params.uuid).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("get_testcase", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Create a new manual test case.")]
    async fn create_testcase(&self, Parameters(params): Parameters<CreateTestcaseParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_testcase", &json!(params));

        let request = CreateTestCaseRequest {
            title: params.title,
            description: params.description,
            project_id: params.project_id,
        };

        let result = self.testmanagement_client.create_testcase(&request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_testcase", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Update an existing test case.")]
    async fn update_testcase(&self, Parameters(params): Parameters<UpdateTestcaseParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("update_testcase", &json!(params));

        let request = UpdateTestCaseRequest {
            title: params.title,
            description: params.description,
            status_code: params.status_code,
        };

        let result = self.testmanagement_client.update_testcase(&params.uuid, &request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("update_testcase", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Delete a test case by UUID.")]
    async fn delete_testcase(&self, Parameters(params): Parameters<UuidParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("delete_testcase", &json!({"uuid": params.uuid}));

        self.testmanagement_client.delete_testcase(&params.uuid).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("delete_testcase", &json!({"deleted": true}));

        Ok(CallToolResult::success(vec![Content::text(json!({"deleted": true, "uuid": params.uuid}).to_string())]))
    }

    #[tool(description = "List test activities with OData filtering.")]
    async fn list_test_activities(&self, Parameters(params): Parameters<ODataListParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_test_activities", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.testmanagement_client.list_activities(query).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_test_activities", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Create a test activity for a test case.")]
    async fn create_test_activity(&self, Parameters(params): Parameters<CreateTestActivityParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_test_activity", &json!(params));

        let request = CreateTestActivityRequest {
            title: params.title,
            parent_id: params.parent_id,
            description: params.description,
            sequence: params.sequence,
        };

        let result = self.testmanagement_client.create_activity(&request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_test_activity", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "List test actions with OData filtering.")]
    async fn list_test_actions(&self, Parameters(params): Parameters<ODataListParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_test_actions", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.testmanagement_client.list_actions(query).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_test_actions", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Create a test action for an activity.")]
    async fn create_test_action(&self, Parameters(params): Parameters<CreateTestActionParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_test_action", &json!(params));

        let request = CreateTestActionRequest {
            title: params.title,
            parent_id: params.parent_id,
            description: params.description,
            expected_result: params.expected_result,
            sequence: params.sequence,
            is_evidence_required: params.is_evidence_required,
        };

        let result = self.testmanagement_client.create_action(&request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_test_action", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    // ========================================================================
    // Process Hierarchy API Tools
    // ========================================================================

    #[tool(description = "List process hierarchy nodes with OData filtering.")]
    async fn list_hierarchy_nodes(&self, Parameters(params): Parameters<ODataListParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_hierarchy_nodes", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.processhierarchy_client.list_nodes(query).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("list_hierarchy_nodes", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Get a hierarchy node by UUID. Optionally expand toParentNode, toChildNodes, toExternalReferences.")]
    async fn get_hierarchy_node(&self, Parameters(params): Parameters<GetHierarchyNodeParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_hierarchy_node", &json!({"uuid": params.uuid, "expand": params.expand}));

        let result = if let Some(ref expand) = params.expand {
            let expand_list: Vec<&str> = expand.split(',').map(|s: &str| s.trim()).collect();
            self.processhierarchy_client.get_node_with_expand(&params.uuid, &expand_list).await
        } else {
            self.processhierarchy_client.get_node(&params.uuid).await
                .map(|n| serde_json::to_value(n).unwrap())
        };

        let json = result.map_err(to_mcp_error)?;
        self.debug.log_tool_result("get_hierarchy_node", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Create a new hierarchy node. Required: title.")]
    async fn create_hierarchy_node(&self, Parameters(params): Parameters<CreateHierarchyNodeParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("create_hierarchy_node", &json!(params));

        let request = CreateHierarchyNodeRequest {
            title: params.title,
            parent_node_uuid: params.parent_node_uuid,
            description: params.description,
            sequence: params.sequence,
        };

        let result = self.processhierarchy_client.create_node(&request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("create_hierarchy_node", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Update an existing hierarchy node.")]
    async fn update_hierarchy_node(&self, Parameters(params): Parameters<UpdateHierarchyNodeParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("update_hierarchy_node", &json!(params));

        let request = UpdateHierarchyNodeRequest {
            title: params.title,
            description: params.description,
            sequence: params.sequence,
        };

        let result = self.processhierarchy_client.update_node(&params.uuid, &request).await
            .map_err(to_mcp_error)?;

        let json = serde_json::to_value(&result).map_err(to_mcp_error)?;
        self.debug.log_tool_result("update_hierarchy_node", &json);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&json).unwrap())]))
    }

    #[tool(description = "Delete a hierarchy node by UUID.")]
    async fn delete_hierarchy_node(&self, Parameters(params): Parameters<UuidParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("delete_hierarchy_node", &json!({"uuid": params.uuid}));

        self.processhierarchy_client.delete_node(&params.uuid).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("delete_hierarchy_node", &json!({"deleted": true}));

        Ok(CallToolResult::success(vec![Content::text(json!({"deleted": true, "uuid": params.uuid}).to_string())]))
    }

    // ========================================================================
    // Analytics API Tools
    // ========================================================================

    #[tool(description = "Query a generic analytics dataset by provider name.")]
    async fn query_analytics_dataset(&self, Parameters(params): Parameters<QueryDatasetParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("query_analytics_dataset", &json!({"provider": params.provider}));

        let query = build_odata_query(
            params.filter,
            params.select,
            None,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.analytics_client.query_dataset(&params.provider, query).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("query_analytics_dataset", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }

    #[tool(description = "List available analytics data providers.")]
    async fn list_analytics_providers(&self) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_analytics_providers", &json!({}));

        let result = self.analytics_client.list_providers().await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("list_analytics_providers", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }

    #[tool(description = "Get requirements analytics data.")]
    async fn get_analytics_requirements(&self, Parameters(params): Parameters<ODataListParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_analytics_requirements", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.analytics_client.get_requirements(query).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("get_analytics_requirements", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }

    #[tool(description = "Get tasks analytics data.")]
    async fn get_analytics_tasks(&self, Parameters(params): Parameters<ODataListParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_analytics_tasks", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.analytics_client.get_tasks_analytics(query).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("get_analytics_tasks", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }

    #[tool(description = "Get alerts analytics data.")]
    async fn get_analytics_alerts(&self, Parameters(params): Parameters<ODataListParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_analytics_alerts", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.analytics_client.get_alerts(query).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("get_analytics_alerts", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }

    // ========================================================================
    // Process Monitoring API Tools
    // ========================================================================

    #[tool(description = "List process monitoring events with OData filtering.")]
    async fn list_monitoring_events(&self, Parameters(params): Parameters<ODataListParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_monitoring_events", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.processmonitoring_client.list_events(query).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("list_monitoring_events", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }

    #[tool(description = "Get a monitoring event by ID.")]
    async fn get_monitoring_event(&self, Parameters(params): Parameters<IdParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_monitoring_event", &json!({"id": params.id}));

        let result = self.processmonitoring_client.get_event(&params.id).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("get_monitoring_event", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }

    #[tool(description = "List monitored services with OData filtering.")]
    async fn list_monitoring_services(&self, Parameters(params): Parameters<ODataListParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("list_monitoring_services", &json!(params));

        let query = build_odata_query(
            params.filter,
            params.select,
            params.expand,
            params.orderby,
            params.top,
            params.skip,
        );

        let result = self.processmonitoring_client.list_services(query).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("list_monitoring_services", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }

    // ========================================================================
    // Logs API Tools
    // ========================================================================

    #[tool(description = "Get logs (outbound) in OpenTelemetry format. Required: provider.")]
    async fn get_logs(&self, Parameters(params): Parameters<GetLogsToolParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("get_logs", &json!(params));

        let log_params = GetLogsParams {
            provider: params.provider,
            format: params.format,
            version: params.version,
            period: params.period,
            from: params.from,
            to: params.to,
            limit: params.limit,
            offset: params.offset,
            service_id: params.service_id,
            observed_timestamp: None,
            on_limit: None,
        };

        let result = self.logs_client.get_logs(&log_params).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("get_logs", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }

    #[tool(description = "Post logs (inbound) in OpenTelemetry format. Required: use_case, service_id, logs.")]
    async fn post_logs(&self, Parameters(params): Parameters<PostLogsToolParams>) -> Result<CallToolResult, McpError> {
        self.debug.log_tool_call("post_logs", &json!({"use_case": params.use_case, "service_id": params.service_id}));

        let log_params = PostLogsParams {
            use_case: params.use_case,
            service_id: params.service_id,
            version: params.version,
            dev: params.dev,
            tag: params.tag,
        };

        let result = self.logs_client.post_logs(&log_params, &params.logs).await
            .map_err(to_mcp_error)?;

        self.debug.log_tool_result("post_logs", &result);

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap())]))
    }
}

// ============================================================================
// Server Handler Implementation
// ============================================================================

#[tool_handler]
impl ServerHandler for SapCloudAlmServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "SAP Cloud ALM MCP Server - Access SAP Cloud ALM APIs for Features, Documents, \
                Tasks, Projects, Test Management, Process Hierarchy, Analytics, Process Monitoring, \
                and Logs.".to_string()
            ),
        }
    }
}
