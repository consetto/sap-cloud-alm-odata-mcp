//! Generic OData v4 client with query builder.

use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth::OAuth2Client;
use crate::error::ApiError;

/// OData query builder for constructing query parameters.
#[derive(Debug, Default, Clone)]
pub struct ODataQuery {
    filter: Option<String>,
    select: Option<Vec<String>>,
    expand: Option<Vec<String>>,
    orderby: Option<Vec<(String, SortOrder)>>,
    top: Option<u32>,
    skip: Option<u32>,
    count: bool,
    search: Option<String>,
}

/// Sort order for $orderby.
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl ODataQuery {
    /// Create a new empty query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a $filter expression.
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Add $select fields.
    pub fn select(mut self, fields: Vec<String>) -> Self {
        self.select = Some(fields);
        self
    }

    /// Add $expand relations.
    pub fn expand(mut self, relations: Vec<String>) -> Self {
        self.expand = Some(relations);
        self
    }

    /// Add $orderby field.
    pub fn orderby(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        let orders = self.orderby.get_or_insert_with(Vec::new);
        orders.push((field.into(), order));
        self
    }

    /// Add $top limit.
    pub fn top(mut self, limit: u32) -> Self {
        self.top = Some(limit);
        self
    }

    /// Add $skip offset.
    pub fn skip(mut self, offset: u32) -> Self {
        self.skip = Some(offset);
        self
    }

    /// Build query string for URL.
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(ref filter) = self.filter {
            params.push(format!("$filter={}", urlencoding::encode(filter)));
        }

        if let Some(ref select) = self.select {
            params.push(format!("$select={}", select.join(",")));
        }

        if let Some(ref expand) = self.expand {
            params.push(format!("$expand={}", expand.join(",")));
        }

        if let Some(ref orderby) = self.orderby {
            let order_str: Vec<String> = orderby
                .iter()
                .map(|(field, order)| {
                    let dir = match order {
                        SortOrder::Asc => "asc",
                        SortOrder::Desc => "desc",
                    };
                    format!("{} {}", field, dir)
                })
                .collect();
            params.push(format!("$orderby={}", order_str.join(",")));
        }

        if let Some(top) = self.top {
            params.push(format!("$top={}", top));
        }

        if let Some(skip) = self.skip {
            params.push(format!("$skip={}", skip));
        }

        if self.count {
            params.push("$count=true".to_string());
        }

        if let Some(ref search) = self.search {
            params.push(format!("$search={}", urlencoding::encode(search)));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// OData v4 collection response wrapper.
#[derive(Debug, Deserialize, Serialize)]
pub struct ODataCollection<T> {
    #[serde(rename = "@odata.context")]
    pub context: Option<String>,

    #[serde(rename = "@odata.count")]
    pub count: Option<i64>,

    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,

    pub value: Vec<T>,
}

/// OData v4 error response.
#[derive(Debug, Deserialize)]
pub struct ODataErrorResponse {
    pub error: ODataErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct ODataErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(default)]
    #[allow(dead_code)]
    details: Vec<ODataErrorItem>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ODataErrorItem {
    code: Option<String>,
    message: String,
    target: Option<String>,
}

/// OData v4 client for SAP Cloud ALM APIs.
#[derive(Clone)]
pub struct ODataClient {
    base_url: String,
    http_client: Client,
    auth_client: OAuth2Client,
    debug: bool,
    is_sandbox: bool,
}

impl ODataClient {
    /// Create a new OData client.
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
    /// Returns ("APIKey", token) for sandbox mode, ("Authorization", "Bearer {token}") for OAuth2.
    fn auth_header(&self, token: &str) -> (&'static str, String) {
        if self.is_sandbox {
            ("APIKey", token.to_string())
        } else {
            ("Authorization", format!("Bearer {}", token))
        }
    }

    /// GET collection with OData query.
    pub async fn get_collection<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<ODataQuery>,
    ) -> Result<ODataCollection<T>, ApiError> {
        let url = format!(
            "{}{}{}",
            self.base_url,
            endpoint,
            query.map(|q| q.to_query_string()).unwrap_or_default()
        );

        self.execute_get(&url).await
    }

    /// GET collection as raw JSON value.
    pub async fn get_collection_raw(
        &self,
        endpoint: &str,
        query: Option<ODataQuery>,
    ) -> Result<Value, ApiError> {
        let url = format!(
            "{}{}{}",
            self.base_url,
            endpoint,
            query.map(|q| q.to_query_string()).unwrap_or_default()
        );

        self.execute_get(&url).await
    }

    /// GET single entity by UUID key.
    pub async fn get_entity_by_uuid<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        uuid: &str,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}/{}", self.base_url, endpoint, uuid);
        self.execute_get(&url).await
    }

    /// GET single entity with expand.
    pub async fn get_entity_with_expand<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        key: &str,
        expand: &[&str],
    ) -> Result<T, ApiError> {
        let expand_str = if expand.is_empty() {
            String::new()
        } else {
            format!("?$expand={}", expand.join(","))
        };
        let url = format!("{}{}/{}{}", self.base_url, endpoint, key, expand_str);
        self.execute_get(&url).await
    }

    /// POST create entity.
    pub async fn create_entity<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, endpoint);
        self.execute_post(&url, body).await
    }

    /// PATCH update entity by UUID.
    pub async fn update_entity_by_uuid<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        uuid: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}/{}", self.base_url, endpoint, uuid);
        self.execute_patch(&url, body).await
    }

    /// DELETE entity by UUID.
    pub async fn delete_entity_by_uuid(&self, endpoint: &str, uuid: &str) -> Result<(), ApiError> {
        let url = format!("{}{}/{}", self.base_url, endpoint, uuid);
        self.execute_delete(&url).await
    }

    /// Execute GET request.
    async fn execute_get<T: DeserializeOwned>(&self, url: &str) -> Result<T, ApiError> {
        if self.debug {
            tracing::debug!(url = %url, "OData GET request");
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

        self.handle_response(response).await
    }

    /// Execute POST request.
    async fn execute_post<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        if self.debug {
            tracing::debug!(url = %url, "OData POST request");
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

        self.handle_response(response).await
    }

    /// Execute PATCH request.
    async fn execute_patch<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        if self.debug {
            tracing::debug!(url = %url, "OData PATCH request");
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

        self.handle_response(response).await
    }

    /// Execute DELETE request.
    async fn execute_delete(&self, url: &str) -> Result<(), ApiError> {
        if self.debug {
            tracing::debug!(url = %url, "OData DELETE request");
        }

        let token = self.auth_client.get_token().await?;
        let (header_name, header_value) = self.auth_header(&token);

        let response = self
            .http_client
            .delete(url)
            .header(header_name, header_value)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status.is_success() || status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            self.parse_error_response(status, &body)
        }
    }

    /// Handle HTTP response and parse JSON.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, ApiError> {
        let status = response.status();

        if status.is_success() {
            let body = response.text().await?;
            if self.debug {
                let truncated = if body.len() > 500 {
                    format!("{}...(truncated)", &body[..500])
                } else {
                    body.clone()
                };
                tracing::debug!(response = %truncated, "OData response received");
            }
            serde_json::from_str(&body).map_err(|e| {
                ApiError::JsonParse(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Failed to parse response: {} - Body: {}",
                        e,
                        &body[..body.len().min(200)]
                    ),
                )))
            })
        } else {
            let body = response.text().await.unwrap_or_default();
            if self.debug {
                tracing::debug!(status = %status, body = %body, "OData error response");
            }
            self.parse_error_response(status, &body)
        }
    }

    /// Parse error response.
    fn parse_error_response<T>(&self, status: StatusCode, body: &str) -> Result<T, ApiError> {
        // Try to parse as OData error
        if let Ok(error) = serde_json::from_str::<ODataErrorResponse>(body) {
            Err(ApiError::ODataError {
                status,
                code: error.error.code,
                message: error.error.message,
            })
        } else {
            Err(ApiError::HttpError {
                status,
                body: body.to_string(),
            })
        }
    }
}

impl std::fmt::Debug for ODataClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ODataClient")
            .field("base_url", &self.base_url)
            .field("debug", &self.debug)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query_returns_empty_string() {
        let query = ODataQuery::new();
        assert_eq!(query.to_query_string(), "");
    }

    #[test]
    fn test_query_with_filter() {
        let query = ODataQuery::new().filter("name eq 'test'");
        let result = query.to_query_string();
        assert!(result.starts_with("?$filter="));
        assert!(result.contains("name%20eq%20%27test%27"));
    }

    #[test]
    fn test_query_with_select() {
        let query = ODataQuery::new().select(vec!["name".to_string(), "id".to_string()]);
        assert_eq!(query.to_query_string(), "?$select=name,id");
    }

    #[test]
    fn test_query_with_expand() {
        let query = ODataQuery::new().expand(vec!["toProject".to_string(), "toStatus".to_string()]);
        assert_eq!(query.to_query_string(), "?$expand=toProject,toStatus");
    }

    #[test]
    fn test_query_with_orderby_asc() {
        let query = ODataQuery::new().orderby("modifiedAt", SortOrder::Asc);
        assert_eq!(query.to_query_string(), "?$orderby=modifiedAt asc");
    }

    #[test]
    fn test_query_with_orderby_desc() {
        let query = ODataQuery::new().orderby("createdAt", SortOrder::Desc);
        assert_eq!(query.to_query_string(), "?$orderby=createdAt desc");
    }

    #[test]
    fn test_query_with_multiple_orderby() {
        let query = ODataQuery::new()
            .orderby("status", SortOrder::Asc)
            .orderby("modifiedAt", SortOrder::Desc);
        assert_eq!(
            query.to_query_string(),
            "?$orderby=status asc,modifiedAt desc"
        );
    }

    #[test]
    fn test_query_with_top() {
        let query = ODataQuery::new().top(10);
        assert_eq!(query.to_query_string(), "?$top=10");
    }

    #[test]
    fn test_query_with_skip() {
        let query = ODataQuery::new().skip(20);
        assert_eq!(query.to_query_string(), "?$skip=20");
    }

    #[test]
    fn test_query_with_pagination() {
        let query = ODataQuery::new().top(10).skip(20);
        assert_eq!(query.to_query_string(), "?$top=10&$skip=20");
    }

    #[test]
    fn test_query_with_multiple_params() {
        let query = ODataQuery::new()
            .filter("projectId eq 'abc'")
            .select(vec!["id".to_string(), "title".to_string()])
            .orderby("modifiedAt", SortOrder::Desc)
            .top(50);
        let result = query.to_query_string();
        assert!(result.contains("$filter="));
        assert!(result.contains("$select=id,title"));
        assert!(
            result.contains("$orderby=modifiedAt%20desc")
                || result.contains("$orderby=modifiedAt desc")
        );
        assert!(result.contains("$top=50"));
    }

    #[test]
    fn test_filter_url_encoding_special_characters() {
        let query = ODataQuery::new().filter("name eq 'O'Reilly & Sons'");
        let result = query.to_query_string();
        // Should encode quotes and ampersand
        assert!(result.contains("%27"));
        assert!(result.contains("%26"));
    }

    #[test]
    fn test_odata_collection_deserialization() {
        let json = r#"{
            "@odata.context": "https://example.com/$metadata#Features",
            "@odata.count": 42,
            "value": [{"id": "1"}, {"id": "2"}]
        }"#;

        #[derive(Debug, Deserialize)]
        struct TestEntity {
            id: String,
        }

        let collection: ODataCollection<TestEntity> = serde_json::from_str(json).unwrap();
        assert_eq!(
            collection.context,
            Some("https://example.com/$metadata#Features".to_string())
        );
        assert_eq!(collection.count, Some(42));
        assert_eq!(collection.value.len(), 2);
        assert_eq!(collection.value[0].id, "1");
    }

    #[test]
    fn test_odata_collection_without_optional_fields() {
        let json = r#"{"value": []}"#;

        #[derive(Debug, Deserialize)]
        struct TestEntity {
            #[allow(dead_code)]
            id: String,
        }

        let collection: ODataCollection<TestEntity> = serde_json::from_str(json).unwrap();
        assert!(collection.context.is_none());
        assert!(collection.count.is_none());
        assert!(collection.next_link.is_none());
        assert!(collection.value.is_empty());
    }

    #[test]
    fn test_odata_error_response_deserialization() {
        let json = r#"{
            "error": {
                "code": "404",
                "message": "Resource not found"
            }
        }"#;

        let error: ODataErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(error.error.code, "404");
        assert_eq!(error.error.message, "Resource not found");
    }

    #[test]
    fn test_sort_order_clone() {
        let asc = SortOrder::Asc;
        let cloned = asc;
        assert!(matches!(cloned, SortOrder::Asc));

        let desc = SortOrder::Desc;
        let cloned = desc;
        assert!(matches!(cloned, SortOrder::Desc));
    }

    #[test]
    fn test_odata_query_clone() {
        let query = ODataQuery::new().filter("test").top(10);
        let cloned = query.clone();
        assert_eq!(cloned.to_query_string(), query.to_query_string());
    }
}
