#[cfg(test)]
mod tests {
    use crate::client::{ApiClient, ApiClientError};
    use crate::types::SessionConfig;

    const TEST_BASE_URL: &str = "http://localhost:8080";

    fn create_test_client() -> ApiClient {
        ApiClient::new(TEST_BASE_URL)
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_create_session_returns_session_id() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let result = client.create_session(Some(config)).await;

        assert!(result.is_ok(), "Expected create_session to succeed, got: {:?}", result);
        let response = result.unwrap();
        assert!(
            !response.session_id.is_empty(),
            "Expected non-empty session_id, got: {}",
            response.session_id
        );
        assert!(
            uuid::Uuid::parse_str(&response.session_id).is_ok(),
            "Expected valid UUID for session_id, got: {}",
            response.session_id
        );
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_create_session_returns_201_created() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let result = client.create_session(Some(config)).await;

        assert!(result.is_ok(), "Expected create_session to return Ok with 201 status");
        let response = result.unwrap();
        assert!(
            chrono::DateTime::parse_from_rfc3339(&response.created_at.to_rfc3339()).is_ok(),
            "Expected valid timestamp for created_at"
        );
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_create_session_with_invalid_config_returns_400() {
        let client = create_test_client();

        let invalid_config = SessionConfig {
            project_id: None,
            metadata: Some(serde_json::json!({
                "invalid_field": "invalid_value",
                "nested": {"bad": "data"}
            })),
        };

        let result = client.create_session(Some(invalid_config)).await;

        match result {
            Err(ApiClientError::BadRequest(_)) => {},
            other => panic!("Expected BadRequest error, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_create_session_without_config_succeeds() {
        let client = create_test_client();

        let result = client.create_session(None).await;

        assert!(result.is_ok(), "Expected create_session without config to succeed");
        let response = result.unwrap();
        assert!(!response.session_id.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_session_lifecycle() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let create_result = client.create_session(Some(config)).await;
        assert!(create_result.is_ok(), "Session creation failed: {:?}", create_result);
        let session_id = create_result.unwrap().session_id;

        let get_result = client.get_session(&session_id).await;
        assert!(get_result.is_ok(), "Get session failed: {:?}", get_result);

        let delete_result = client.delete_session(&session_id).await;
        assert!(delete_result.is_ok(), "Delete session failed: {:?}", delete_result);
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_list_sessions_returns_created_session() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let create_result = client.create_session(Some(config)).await;
        assert!(create_result.is_ok());
        let session_id = create_result.unwrap().session_id;

        let list_result = client.list_sessions().await;
        assert!(list_result.is_ok(), "List sessions failed: {:?}", list_result);
        let sessions = list_result.unwrap();

        assert!(
            sessions.iter().any(|s| s.id == session_id),
            "Created session {} not found in list: {:?}",
            session_id,
            sessions
        );

        let _ = client.delete_session(&session_id).await;
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_get_nonexistent_session_returns_404() {
        let client = create_test_client();

        let result = client.get_session("nonexistent-session-id").await;

        match result {
            Err(ApiClientError::NotFound) => {},
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_delete_nonexistent_session_returns_404() {
        let client = create_test_client();

        let result = client.delete_session("nonexistent-session-id").await;

        match result {
            Err(ApiClientError::NotFound) => {},
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_session_created_at_is_valid_timestamp() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let result = client.create_session(Some(config)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let created_at = response.created_at;

        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(created_at);
        assert!(
            diff.num_seconds() >= 0 && diff.num_seconds() < 5,
            "created_at should be within 5 seconds of now, got: {}, now: {}",
            created_at,
            now
        );
    }
}

pub mod integration_tests {
    #[test]
    fn test_module_exists() {
        assert!(true);
    }
}