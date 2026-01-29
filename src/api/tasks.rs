//! Tasks API client (REST) - CALM_TKM.
//! Note: This is a REST API, not OData.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::auth::OAuth2Client;
use crate::error::ApiError;

/// Task entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: Option<String>,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub task_type: Option<String>,
    pub status: Option<String>,
    pub sub_status: Option<String>,
    pub external_id: Option<String>,
    pub due_date: Option<String>,
    pub priority_id: Option<i32>,
    pub assignee_id: Option<String>,
    pub assignee_name: Option<String>,
    pub timebox_name: Option<String>,
    pub timebox_start_date: Option<String>,
    pub timebox_end_date: Option<String>,
    pub last_changed_date: Option<String>,
}

/// Task comment entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskComment {
    pub id: Option<String>,
    pub task_id: Option<String>,
    pub content: Option<String>,
    pub created_at: Option<String>,
    pub created_by: Option<String>,
}

/// Task reference entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskReference {
    pub id: Option<String>,
    pub task_id: Option<String>,
    pub external_id: Option<String>,
    pub external_system: Option<String>,
    pub url: Option<String>,
}

/// Workstream entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workstream {
    pub id: Option<String>,
    pub project_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Deliverable entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deliverable {
    pub id: Option<String>,
    pub project_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request to create a task.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub project_id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub task_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
}

/// Request to update a task.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
}

/// Request to create a task comment.
#[derive(Debug, Clone, Serialize)]
pub struct CreateTaskCommentRequest {
    pub content: String,
}

/// Query parameters for listing tasks.
#[derive(Debug, Clone, Default)]
pub struct ListTasksParams {
    pub project_id: String,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub task_type: Option<String>,
    pub status: Option<String>,
    pub sub_status: Option<String>,
    pub assignee_id: Option<String>,
    pub last_changed_date: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Tasks API client.
#[derive(Clone)]
pub struct TasksClient {
    base_url: String,
    http_client: Client,
    auth_client: OAuth2Client,
    debug: bool,
    is_sandbox: bool,
}

impl TasksClient {
    /// Create a new Tasks client.
    ///
    /// # Errors
    /// Returns `ApiError::HttpClientInit` if the HTTP client cannot be created.
    pub fn new(base_url: String, auth_client: OAuth2Client, debug: bool) -> Result<Self, ApiError> {
        let is_sandbox = auth_client.is_sandbox();
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ApiError::HttpClientInit(e.to_string()))?;

        Ok(Self {
            base_url,
            http_client,
            auth_client,
            debug,
            is_sandbox,
        })
    }

    /// Get the appropriate auth header name and value.
    fn auth_header(&self, token: &str) -> (&'static str, String) {
        if self.is_sandbox {
            ("APIKey", token.to_string())
        } else {
            ("Authorization", format!("Bearer {}", token))
        }
    }

    /// List tasks for a project.
    pub async fn list_tasks(&self, params: &ListTasksParams) -> Result<Vec<Task>, ApiError> {
        let mut url = format!("{}/tasks?projectId={}", self.base_url, params.project_id);

        if let Some(offset) = params.offset {
            url.push_str(&format!("&offset={}", offset));
        }
        if let Some(limit) = params.limit {
            url.push_str(&format!("&limit={}", limit));
        }
        if let Some(ref t) = params.task_type {
            url.push_str(&format!("&type={}", t));
        }
        if let Some(ref s) = params.status {
            url.push_str(&format!("&status={}", s));
        }
        if let Some(ref ss) = params.sub_status {
            url.push_str(&format!("&subStatus={}", ss));
        }
        if let Some(ref a) = params.assignee_id {
            url.push_str(&format!("&assigneeId={}", a));
        }
        if let Some(ref d) = params.last_changed_date {
            url.push_str(&format!("&lastChangedDate={}", d));
        }
        if let Some(ref tags) = params.tags {
            for tag in tags {
                url.push_str(&format!("&tags={}", urlencoding::encode(tag)));
            }
        }

        self.get(&url).await
    }

    /// Get a single task by ID.
    pub async fn get_task(&self, id: &str) -> Result<Task, ApiError> {
        let url = format!("{}/tasks/{}", self.base_url, id);
        self.get(&url).await
    }

    /// Create a new task.
    pub async fn create_task(&self, request: &CreateTaskRequest) -> Result<Task, ApiError> {
        let url = format!("{}/tasks", self.base_url);
        self.post(&url, request).await
    }

    /// Update an existing task.
    pub async fn update_task(
        &self,
        id: &str,
        request: &UpdateTaskRequest,
    ) -> Result<Task, ApiError> {
        let url = format!("{}/tasks/{}", self.base_url, id);
        self.patch(&url, request).await
    }

    /// Delete a task.
    pub async fn delete_task(&self, id: &str) -> Result<(), ApiError> {
        let url = format!("{}/tasks/{}", self.base_url, id);
        self.delete(&url).await
    }

    /// List comments for a task.
    pub async fn list_task_comments(&self, task_id: &str) -> Result<Vec<TaskComment>, ApiError> {
        let url = format!("{}/tasks/{}/comments", self.base_url, task_id);
        self.get(&url).await
    }

    /// Create a comment on a task.
    pub async fn create_task_comment(
        &self,
        task_id: &str,
        request: &CreateTaskCommentRequest,
    ) -> Result<TaskComment, ApiError> {
        let url = format!("{}/tasks/{}/comments", self.base_url, task_id);
        self.post(&url, request).await
    }

    /// List references for a task.
    pub async fn list_task_references(
        &self,
        task_id: &str,
    ) -> Result<Vec<TaskReference>, ApiError> {
        let url = format!("{}/tasks/{}/references", self.base_url, task_id);
        self.get(&url).await
    }

    /// List workstreams for a project.
    pub async fn list_workstreams(&self, project_id: &str) -> Result<Vec<Workstream>, ApiError> {
        let url = format!("{}/workstreams?projectId={}", self.base_url, project_id);
        self.get(&url).await
    }

    /// List deliverables for a project.
    pub async fn list_deliverables(&self, project_id: &str) -> Result<Vec<Deliverable>, ApiError> {
        let url = format!("{}/deliverables?projectId={}", self.base_url, project_id);
        self.get(&url).await
    }

    /// Execute GET request.
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, ApiError> {
        if self.debug {
            eprintln!("[TASKS] GET {}", url);
        }

        let token = self.auth_client.get_token().await?;
        let (header_name, header_value) = self.auth_header(&token);

        let response = self
            .http_client
            .get(url)
            .header(header_name, header_value)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::HttpError { status, body })
        }
    }

    /// Execute POST request.
    async fn post<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        if self.debug {
            eprintln!("[TASKS] POST {}", url);
        }

        let token = self.auth_client.get_token().await?;
        let (header_name, header_value) = self.auth_header(&token);

        let response = self
            .http_client
            .post(url)
            .header(header_name, header_value)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::HttpError { status, body })
        }
    }

    /// Execute PATCH request.
    async fn patch<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        if self.debug {
            eprintln!("[TASKS] PATCH {}", url);
        }

        let token = self.auth_client.get_token().await?;
        let (header_name, header_value) = self.auth_header(&token);

        let response = self
            .http_client
            .patch(url)
            .header(header_name, header_value)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::HttpError { status, body })
        }
    }

    /// Execute DELETE request.
    async fn delete(&self, url: &str) -> Result<(), ApiError> {
        if self.debug {
            eprintln!("[TASKS] DELETE {}", url);
        }

        let token = self.auth_client.get_token().await?;
        let (header_name, header_value) = self.auth_header(&token);

        let response = self
            .http_client
            .delete(url)
            .header(header_name, header_value)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() || status == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::HttpError { status, body })
        }
    }
}

impl std::fmt::Debug for TasksClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TasksClient")
            .field("base_url", &self.base_url)
            .finish()
    }
}
