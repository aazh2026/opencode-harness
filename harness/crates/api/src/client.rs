use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::types::*;

pub struct ApiClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self {
            client,
            base_url: base_url.into(),
            auth_token: None,
        }
    }

    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    pub async fn create_session(
        &self,
        config: Option<SessionConfig>,
    ) -> Result<CreateSessionResponse, ApiClientError> {
        let url = format!("{}/session", self.base_url);
        let mut request = self.client.post(&url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let request_body = config.map(|c| CreateSessionRequest {
            project_id: c.project_id,
            metadata: c.metadata,
        });

        let response = request
            .json(&request_body)
            .send()
            .await
            .map_err(ApiClientError::Request)?;

        let status = response.status();
        if status.is_success() {
            let session: Session = response
                .json()
                .await
                .map_err(ApiClientError::Response)?;
            Ok(CreateSessionResponse::from(session))
        } else if status.as_u16() == 400 {
            Err(ApiClientError::BadRequest(
                response
                    .json()
                    .await
                    .map_err(ApiClientError::Response)?,
            ))
        } else if status.as_u16() == 401 {
            Err(ApiClientError::Unauthorized)
        } else {
            Err(ApiClientError::UnexpectedStatus(status))
        }
    }

    pub async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<Session, ApiClientError> {
        let url = format!("{}/session/{}", self.base_url, session_id);
        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(ApiClientError::Request)?;

        let status = response.status();
        if status.is_success() {
            response
                .json()
                .await
                .map_err(ApiClientError::Response)
        } else if status.as_u16() == 404 {
            Err(ApiClientError::NotFound)
        } else if status.as_u16() == 401 {
            Err(ApiClientError::Unauthorized)
        } else {
            Err(ApiClientError::UnexpectedStatus(status))
        }
    }

    pub async fn list_sessions(&self) -> Result<Vec<Session>, ApiClientError> {
        let url = format!("{}/session", self.base_url);
        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(ApiClientError::Request)?;

        let status = response.status();
        if status.is_success() {
            #[derive(Deserialize)]
            struct SessionsResponse { sessions: Vec<Session> }
            let result: SessionsResponse = response
                .json()
                .await
                .map_err(ApiClientError::Response)?;
            Ok(result.sessions)
        } else if status.as_u16() == 401 {
            Err(ApiClientError::Unauthorized)
        } else {
            Err(ApiClientError::UnexpectedStatus(status))
        }
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), ApiClientError> {
        let url = format!("{}/session/{}", self.base_url, session_id);
        let mut request = self.client.delete(&url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(ApiClientError::Request)?;

        let status = response.status();
        if status.is_success() || status.as_u16() == 204 {
            Ok(())
        } else if status.as_u16() == 404 {
            Err(ApiClientError::NotFound)
        } else if status.as_u16() == 401 {
            Err(ApiClientError::Unauthorized)
        } else {
            Err(ApiClientError::UnexpectedStatus(status))
        }
    }

    pub async fn resume_session(
        &self,
        session_id: &str,
    ) -> Result<ResumeSessionResponse, ApiClientError> {
        let url = format!("{}/sessions/{}/resume", self.base_url, session_id);
        let mut request = self.client.post(&url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(ApiClientError::Request)?;

        let status = response.status();
        if status.is_success() {
            response
                .json()
                .await
                .map_err(ApiClientError::Response)
        } else if status.as_u16() == 404 {
            Err(ApiClientError::NotFound)
        } else if status.as_u16() == 401 {
            Err(ApiClientError::Unauthorized)
        } else if status.as_u16() == 410 {
            Err(ApiClientError::Gone)
        } else {
            Err(ApiClientError::UnexpectedStatus(status))
        }
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, ApiClientError> {
        let url = format!("{}/projects", self.base_url);
        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(ApiClientError::Request)?;

        let status = response.status();
        if status.is_success() {
            #[derive(Deserialize)]
            struct ProjectsResponse { projects: Vec<Project> }
            let result: ProjectsResponse = response
                .json()
                .await
                .map_err(ApiClientError::Response)?;
            Ok(result.projects)
        } else if status.as_u16() == 401 {
            Err(ApiClientError::Unauthorized)
        } else {
            Err(ApiClientError::UnexpectedStatus(status))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiClientError {
    #[error("Request failed: {0}")]
    Request(#[source] reqwest::Error),
    #[error("Response parsing failed: {0}")]
    Response(#[source] reqwest::Error),
    #[error("Bad request: {0:?}")]
    BadRequest(ApiError),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Not found")]
    NotFound,
    #[error("Gone")]
    Gone,
    #[error("Unexpected status: {0}")]
    UnexpectedStatus(reqwest::StatusCode),
}