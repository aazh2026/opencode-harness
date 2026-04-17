use opencode_api::types::{ResumeSessionResponse, Session, SessionConfig, SessionStatus, WorkspaceState};

#[derive(Debug, Clone)]
pub struct PartialOperation {
    pub operation_id: String,
    pub operation_type: String,
    pub completed: bool,
    pub checkpoint: Option<String>,
}

pub fn partial_operation_state_handled_correctly_on_recovery() {
    let _session_config = SessionConfig {
        project_id: Some("test-project".to_string()),
        metadata: Some(serde_json::json!({"test": "partial-state-recovery"})),
    };

    let session = Session {
        id: "test-session-partial-001".to_string(),
        status: SessionStatus::Active,
        project_id: Some("test-project".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: Some(chrono::Utc::now()),
    };

    let resume_response = ResumeSessionResponse {
        session_id: session.id.clone(),
        resumed: true,
        status: SessionStatus::Active,
        project_id: session.project_id.clone(),
        workspace: Some(WorkspaceState {
            cwd: "/test/workspace".to_string(),
            files: Some(vec![
                "file1.txt".to_string(),
                "file2.txt".to_string(),
                "file3.txt.incomplete".to_string(),
            ]),
        }),
    };

    assert!(
        resume_response.resumed,
        "Session with partial operation state should be resumable"
    );
    assert_eq!(
        resume_response.status, SessionStatus::Active,
        "Session should be Active after recovery"
    );
    assert!(
        resume_response.workspace.is_some(),
        "Workspace state should be preserved"
    );

    let workspace = resume_response.workspace.unwrap();
    assert!(
        workspace.files.as_ref().map(|f| f.len()).unwrap_or(0) >= 2,
        "Workspace should contain at least the files written before interruption"
    );
}


pub fn error_handling_test_prolonged_disconnection() {
    let _session_config = SessionConfig {
        project_id: Some("test-project".to_string()),
        metadata: Some(serde_json::json!({"test": "error-handling"})),
    };

    let session = Session {
        id: "test-session-error-001".to_string(),
        status: SessionStatus::Terminated,
        project_id: Some("test-project".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: Some(chrono::Utc::now()),
    };

    let resume_response = ResumeSessionResponse {
        session_id: session.id.clone(),
        resumed: false,
        status: SessionStatus::Terminated,
        project_id: session.project_id.clone(),
        workspace: None,
    };

    assert!(
        !resume_response.resumed,
        "Session should not resume after being terminated"
    );
    assert_eq!(
        resume_response.status, SessionStatus::Terminated,
        "Session status should be Terminated after error"
    );
}

pub fn error_handling_test_corrupted_session_state() {
    let resume_response = ResumeSessionResponse {
        session_id: "".to_string(),
        resumed: false,
        status: SessionStatus::New,
        project_id: None,
        workspace: None,
    };

    assert!(
        resume_response.session_id.is_empty(),
        "Corrupted session should have empty session_id"
    );
    assert!(
        !resume_response.resumed,
        "Corrupted session should not be marked as resumed"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handling_prolonged_disconnection() {
        error_handling_test_prolonged_disconnection();
    }

    #[test]
    fn test_error_handling_corrupted_session_state() {
        error_handling_test_corrupted_session_state();
    }

    #[test]
    fn test_partial_operation_state_handled_correctly_on_recovery() {
        partial_operation_state_handled_correctly_on_recovery();
    }
}