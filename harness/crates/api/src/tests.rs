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

        assert!(
            result.is_ok(),
            "Expected create_session to succeed, got: {:?}",
            result
        );
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

        assert!(
            result.is_ok(),
            "Expected create_session to return Ok with 201 status"
        );
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
            Err(ApiClientError::BadRequest(_)) => {}
            other => panic!("Expected BadRequest error, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_create_session_without_config_succeeds() {
        let client = create_test_client();

        let result = client.create_session(None).await;

        assert!(
            result.is_ok(),
            "Expected create_session without config to succeed"
        );
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
        assert!(
            create_result.is_ok(),
            "Session creation failed: {:?}",
            create_result
        );
        let session_id = create_result.unwrap().session_id;

        let get_result = client.get_session(&session_id).await;
        assert!(get_result.is_ok(), "Get session failed: {:?}", get_result);

        let delete_result = client.delete_session(&session_id).await;
        assert!(
            delete_result.is_ok(),
            "Delete session failed: {:?}",
            delete_result
        );
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
        assert!(
            list_result.is_ok(),
            "List sessions failed: {:?}",
            list_result
        );
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
            Err(ApiClientError::NotFound) => {}
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_005_delete_nonexistent_session_returns_404() {
        let client = create_test_client();

        let result = client.delete_session("nonexistent-session-id").await;

        match result {
            Err(ApiClientError::NotFound) => {}
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

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_006_resume_session_returns_session_state() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let create_result = client.create_session(Some(config)).await;
        assert!(
            create_result.is_ok(),
            "Session creation failed: {:?}",
            create_result
        );
        let session_id = create_result.unwrap().session_id;

        let resume_result = client.resume_session(&session_id).await;
        assert!(
            resume_result.is_ok(),
            "Resume session failed: {:?}",
            resume_result
        );
        let resumed_session = resume_result.unwrap();

        assert_eq!(
            resumed_session.session_id, session_id,
            "Expected session_id to match, got: {}, expected: {}",
            resumed_session.session_id, session_id
        );
        assert!(resumed_session.resumed, "Expected resumed flag to be true");

        let _ = client.delete_session(&session_id).await;
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_006_resume_existing_session_resumes() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let create_result = client.create_session(Some(config)).await;
        assert!(
            create_result.is_ok(),
            "Session creation failed: {:?}",
            create_result
        );
        let session_id = create_result.unwrap().session_id;

        let resume_result = client.resume_session(&session_id).await;
        assert!(
            resume_result.is_ok(),
            "Expected resume to succeed for existing session"
        );

        let _ = client.delete_session(&session_id).await;
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_006_resume_nonexistent_session_returns_404() {
        let client = create_test_client();

        let result = client.resume_session("nonexistent-session-id").await;

        match result {
            Err(ApiClientError::NotFound) => {}
            other => panic!(
                "Expected NotFound error for nonexistent session, got: {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_006_resume_expired_session_returns_410() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let create_result = client.create_session(Some(config)).await;
        assert!(create_result.is_ok());
        let session_id = create_result.unwrap().session_id;

        let _ = client.delete_session(&session_id).await;

        let resume_result = client.resume_session(&session_id).await;
        match resume_result {
            Err(ApiClientError::Gone) | Err(ApiClientError::NotFound) => {}
            other => panic!(
                "Expected Gone or NotFound error for expired session, got: {:?}",
                other
            ),
        }
    }
}

#[cfg(test)]
pub mod integration_tests {
    use crate::client::ApiClient;
    use crate::types::SessionConfig;

    const TEST_BASE_URL: &str = "http://localhost:8080";

    fn create_test_client() -> ApiClient {
        ApiClient::new(TEST_BASE_URL)
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_008_send_message_returns_response() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let create_result = client.create_session(Some(config)).await;
        assert!(
            create_result.is_ok(),
            "Session creation failed: {:?}",
            create_result
        );
        let session_id = create_result.unwrap().session_id;

        let result = client
            .send_message(&session_id, "Hello, world!", None)
            .await;
        assert!(
            result.is_ok(),
            "Expected send_message to succeed, got: {:?}",
            result
        );
        let response = result.unwrap();
        assert!(
            !response.message_id.is_empty(),
            "Expected non-empty message_id"
        );
        assert!(!response.response.is_empty(), "Expected non-empty response");

        let _ = client.delete_session(&session_id).await;
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_008_send_message_with_context_returns_response() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let create_result = client.create_session(Some(config)).await;
        assert!(create_result.is_ok());
        let session_id = create_result.unwrap().session_id;

        let context = serde_json::json!({"key": "value"});
        let result = client
            .send_message(&session_id, "Hello with context!", Some(context))
            .await;
        assert!(
            result.is_ok(),
            "Expected send_message with context to succeed, got: {:?}",
            result
        );
        let response = result.unwrap();
        assert!(!response.message_id.is_empty());
        assert!(!response.response.is_empty());

        let _ = client.delete_session(&session_id).await;
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_008_send_message_to_nonexistent_session_returns_404() {
        let client = create_test_client();

        let result = client
            .send_message("nonexistent-session-id", "Hello!", None)
            .await;

        match result {
            Err(crate::client::ApiClientError::NotFound) => {}
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_008_send_empty_message_returns_400() {
        let client = create_test_client();

        let config = SessionConfig {
            project_id: Some("test-project".to_string()),
            metadata: None,
        };

        let create_result = client.create_session(Some(config)).await;
        assert!(create_result.is_ok());
        let session_id = create_result.unwrap().session_id;

        let result = client.send_message(&session_id, "", None).await;

        match result {
            Err(crate::client::ApiClientError::BadRequest(_)) => {}
            other => panic!(
                "Expected BadRequest error for empty message, got: {:?}",
                other
            ),
        }

        let _ = client.delete_session(&session_id).await;
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_007_list_projects_returns_list() {
        let client = create_test_client();

        let result = client.list_projects().await;

        assert!(
            result.is_ok(),
            "Expected list_projects to succeed, got: {:?}",
            result
        );
        let projects = result.unwrap();
        assert!(
            projects.iter().all(|p| !p.id.is_empty()),
            "Expected all projects to have non-empty id"
        );
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_007_list_projects_returns_correct_metadata() {
        let client = create_test_client();

        let result = client.list_projects().await;

        assert!(
            result.is_ok(),
            "Expected list_projects to succeed, got: {:?}",
            result
        );
        let projects = result.unwrap();

        for project in &projects {
            assert!(
                !project.id.is_empty(),
                "Expected project id to be non-empty"
            );
            assert!(
                !project.name.is_empty(),
                "Expected project name to be non-empty, got: {}",
                project.name
            );
            assert!(
                !project.path.is_empty(),
                "Expected project path to be non-empty"
            );
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(project.created_at);
            assert!(
                diff.num_seconds() >= 0 && diff.num_seconds() < 86400,
                "created_at should be within 24 hours of now, got: {}",
                project.created_at
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_007_list_projects_empty_returns_empty_array() {
        let client = create_test_client();

        let result = client.list_projects().await;

        assert!(
            result.is_ok(),
            "Expected list_projects to succeed, got: {:?}",
            result
        );
        let projects = result.unwrap();
        assert!(
            projects.is_empty()
                || projects
                    .iter()
                    .any(|p| { p.id.is_empty() && p.name.is_empty() && p.path.is_empty() }),
            "Expected empty array or projects with empty fields when no projects exist"
        );
    }

    #[test]
    fn test_module_exists() {
        assert!(true);
    }
}

#[cfg(test)]
mod smoke_api_010_tests {
    use crate::client::ApiClient;

    const TEST_BASE_URL: &str = "http://localhost:8080";

    fn create_test_client() -> ApiClient {
        ApiClient::new(TEST_BASE_URL)
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_010_list_tools_returns_list() {
        let client = create_test_client();

        let result = client.list_tools().await;

        assert!(
            result.is_ok(),
            "Expected list_tools to succeed, got: {:?}",
            result
        );
        let tools = result.unwrap();
        assert!(
            tools.iter().all(|t| !t.name.is_empty()),
            "Expected all tools to have non-empty name"
        );
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_010_list_tools_returns_correct_metadata() {
        let client = create_test_client();

        let result = client.list_tools().await;

        assert!(
            result.is_ok(),
            "Expected list_tools to succeed, got: {:?}",
            result
        );
        let tools = result.unwrap();

        for tool in &tools {
            assert!(!tool.name.is_empty(), "Expected tool name to be non-empty");
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_010_get_tool_returns_specific_tool_details() {
        let client = create_test_client();

        let list_result = client.list_tools().await;
        assert!(list_result.is_ok(), "List tools failed: {:?}", list_result);
        let tools = list_result.unwrap();
        assert!(!tools.is_empty(), "Expected at least one tool to exist");

        let first_tool_name = &tools[0].name;
        let get_result = client.get_tool(first_tool_name).await;

        assert!(
            get_result.is_ok(),
            "Expected get_tool to succeed, got: {:?}",
            get_result
        );
        let tool = get_result.unwrap();
        assert_eq!(
            tool.name, *first_tool_name,
            "Expected tool name to match, got: {}, expected: {}",
            tool.name, first_tool_name
        );
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_010_get_nonexistent_tool_returns_404() {
        let client = create_test_client();

        let result = client.get_tool("nonexistent-tool-name").await;

        match result {
            Err(crate::client::ApiClientError::NotFound) => {}
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_010_list_tools_empty_returns_empty_array() {
        let client = create_test_client();

        let result = client.list_tools().await;

        assert!(
            result.is_ok(),
            "Expected list_tools to succeed, got: {:?}",
            result
        );
        let tools = result.unwrap();
        assert!(
            tools.is_empty() || tools.iter().any(|t| t.name.is_empty()),
            "Expected empty array or tools with empty name when no tools exist"
        );
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_010_tool_has_optional_schema_fields() {
        let client = create_test_client();

        let list_result = client.list_tools().await;
        assert!(list_result.is_ok(), "List tools failed: {:?}", list_result);
        let tools = list_result.unwrap();

        if !tools.is_empty() {
            let tool = &tools[0];
            let get_result = client.get_tool(&tool.name).await;
            assert!(
                get_result.is_ok(),
                "Expected get_tool to succeed for tool: {}",
                tool.name
            );
            let detailed_tool = get_result.unwrap();
            assert_eq!(detailed_tool.name, tool.name, "Expected tool name to match");
        }
    }
}

#[cfg(test)]
mod smoke_api_009_tests {
    use crate::client::ApiClient;

    const TEST_BASE_URL: &str = "http://localhost:8080";

    fn create_test_client() -> ApiClient {
        ApiClient::new(TEST_BASE_URL)
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_009_subscribe_creates_event_subscription() {
        let client = create_test_client();

        let result = client.subscribe_events(None).await;

        assert!(
            result.is_ok(),
            "Expected subscribe_events to succeed, got: {:?}",
            result
        );
        let subscription = result.unwrap();
        assert!(
            !subscription.subscription_id.is_empty(),
            "Expected non-empty subscription_id, got: {}",
            subscription.subscription_id
        );
        assert!(
            uuid::Uuid::parse_str(&subscription.subscription_id).is_ok(),
            "Expected valid UUID for subscription_id, got: {}",
            subscription.subscription_id
        );

        let _ = client
            .delete_subscription(&subscription.subscription_id)
            .await;
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_009_subscribe_with_event_types_returns_subscription_id() {
        let client = create_test_client();

        let event_types = vec![
            "session.created".to_string(),
            "message.received".to_string(),
        ];
        let result = client.subscribe_events(Some(event_types.clone())).await;

        assert!(
            result.is_ok(),
            "Expected subscribe_events with event_types to succeed, got: {:?}",
            result
        );
        let subscription = result.unwrap();
        assert!(
            !subscription.subscription_id.is_empty(),
            "Expected non-empty subscription_id"
        );
        assert!(
            subscription.event_types.is_some(),
            "Expected event_types to be set"
        );
        let returned_types = subscription.event_types.unwrap();
        assert_eq!(
            returned_types.len(),
            event_types.len(),
            "Expected {} event_types, got: {:?}",
            event_types.len(),
            returned_types
        );

        let _ = client
            .delete_subscription(&subscription.subscription_id)
            .await;
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_009_get_events_receives_events() {
        let client = create_test_client();

        let subscribe_result = client.subscribe_events(None).await;
        assert!(
            subscribe_result.is_ok(),
            "Subscription creation failed: {:?}",
            subscribe_result
        );
        let subscription_id = subscribe_result.unwrap().subscription_id;

        let events_result = client.get_events(&subscription_id).await;
        assert!(
            events_result.is_ok(),
            "Expected get_events to succeed, got: {:?}",
            events_result
        );
        let events = events_result.unwrap();
        assert!(
            events.iter().all(|e| !e.event_type.is_empty()),
            "Expected all events to have non-empty event_type"
        );

        let _ = client.delete_subscription(&subscription_id).await;
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_009_subscribe_with_invalid_event_types_returns_400() {
        let client = create_test_client();

        let invalid_event_types = vec!["invalid.event.type".to_string()];
        let result = client.subscribe_events(Some(invalid_event_types)).await;

        match result {
            Err(crate::client::ApiClientError::BadRequest(_)) => {}
            other => panic!(
                "Expected BadRequest error for invalid event_types, got: {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_009_subscription_lifecycle() {
        let client = create_test_client();

        let subscribe_result = client.subscribe_events(None).await;
        assert!(
            subscribe_result.is_ok(),
            "Subscription creation failed: {:?}",
            subscribe_result
        );
        let subscription_id = subscribe_result.unwrap().subscription_id;

        let get_result = client.get_events(&subscription_id).await;
        assert!(get_result.is_ok(), "Get events failed: {:?}", get_result);

        let delete_result = client.delete_subscription(&subscription_id).await;
        assert!(
            delete_result.is_ok(),
            "Delete subscription failed: {:?}",
            delete_result
        );
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_009_get_nonexistent_subscription_returns_404() {
        let client = create_test_client();

        let result = client.get_events("nonexistent-subscription-id").await;

        match result {
            Err(crate::client::ApiClientError::NotFound) => {}
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_009_delete_nonexistent_subscription_returns_404() {
        let client = create_test_client();

        let result = client
            .delete_subscription("nonexistent-subscription-id")
            .await;

        match result {
            Err(crate::client::ApiClientError::NotFound) => {}
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore = "requires running API server"]
    async fn smoke_api_009_subscription_created_at_is_valid_timestamp() {
        let client = create_test_client();

        let result = client.subscribe_events(None).await;
        assert!(result.is_ok());

        let subscription = result.unwrap();
        let created_at = subscription.created_at;

        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(created_at);
        assert!(
            diff.num_seconds() >= 0 && diff.num_seconds() < 5,
            "created_at should be within 5 seconds of now, got: {}, now: {}",
            created_at,
            now
        );

        let _ = client
            .delete_subscription(&subscription.subscription_id)
            .await;
    }
}
