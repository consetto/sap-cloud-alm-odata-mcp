//! Projects API client (REST) - CALM_PJM.
//! Note: This is a REST API, not OData.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::auth::OAuth2Client;
use crate::error::ApiError;

/// Project entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub project_type: Option<String>,
    pub program_id: Option<String>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
}

/// Program entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Timebox (sprint) entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timebox {
    pub id: Option<String>,
    pub name: Option<String>,
    pub project_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
}

/// Team member entity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub project_id: Option<String>,
}

/// Request to create a project.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_id: Option<String>,
}

/// Projects API client.
#[derive(Clone)]
pub struct ProjectsClient {
    base_url: String,
    http_client: Client,
    auth_client: OAuth2Client,
    debug: bool,
    is_sandbox: bool,
}

impl ProjectsClient {
    /// Create a new Projects client.
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

    /// List all projects.
    pub async fn list_projects(&self) -> Result<Vec<Project>, ApiError> {
        let url = format!("{}/projects", self.base_url);
        self.get(&url).await
    }

    /// Get a single project by ID.
    pub async fn get_project(&self, id: &str) -> Result<Project, ApiError> {
        let url = format!("{}/projects/{}", self.base_url, id);
        self.get(&url).await
    }

    /// Create a new project.
    pub async fn create_project(
        &self,
        request: &CreateProjectRequest,
    ) -> Result<Project, ApiError> {
        let url = format!("{}/projects", self.base_url);
        self.post(&url, request).await
    }

    /// List timeboxes (sprints) for a project.
    pub async fn list_timeboxes(&self, project_id: &str) -> Result<Vec<Timebox>, ApiError> {
        let url = format!("{}/projects/{}/timeboxes", self.base_url, project_id);
        self.get(&url).await
    }

    /// List team members for a project.
    pub async fn list_team_members(&self, project_id: &str) -> Result<Vec<TeamMember>, ApiError> {
        let url = format!("{}/projects/{}/teams", self.base_url, project_id);
        self.get(&url).await
    }

    /// List all programs.
    pub async fn list_programs(&self) -> Result<Vec<Program>, ApiError> {
        let url = format!("{}/programs", self.base_url);
        self.get(&url).await
    }

    /// Get a single program by ID.
    pub async fn get_program(&self, id: &str) -> Result<Program, ApiError> {
        let url = format!("{}/programs/{}", self.base_url, id);
        self.get(&url).await
    }

    /// Execute GET request.
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, ApiError> {
        if self.debug {
            tracing::debug!(url = %url, "Projects API GET request");
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
            tracing::debug!(url = %url, "Projects API POST request");
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
}

impl std::fmt::Debug for ProjectsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectsClient")
            .field("base_url", &self.base_url)
            .finish()
    }
}
