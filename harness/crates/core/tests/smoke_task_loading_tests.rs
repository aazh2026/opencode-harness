use opencode_core::loaders::task_loader::{DefaultTaskLoader, TaskLoader};
use opencode_core::loaders::task_validator::{DefaultTaskSchemaValidator, TaskSchemaValidator};
use opencode_core::types::entry_mode::EntryMode;
use opencode_core::types::task::TaskCategory;
use std::path::PathBuf;

fn get_tasks_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../../harness/tasks");
    path
}

fn get_cli_tasks_dir() -> PathBuf {
    let mut path = get_tasks_dir();
    path.push("cli");
    path
}

fn get_workspace_tasks_dir() -> PathBuf {
    let mut path = get_tasks_dir();
    path.push("workspace");
    path
}

fn get_session_tasks_dir() -> PathBuf {
    let mut path = get_tasks_dir();
    path.push("session");
    path
}

fn get_permissions_tasks_dir() -> PathBuf {
    let mut path = get_tasks_dir();
    path.push("permissions");
    path
}

fn get_api_tasks_dir() -> PathBuf {
    let mut path = get_tasks_dir();
    path.push("api");
    path
}

fn get_recovery_tasks_dir() -> PathBuf {
    let mut path = get_tasks_dir();
    path.push("recovery");
    path
}

mod smoke_cli_tests {
    use super::*;

    #[test]
    fn task_loader_can_load_all_smoke_cli_tasks() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_cli_tasks_dir())
            .expect("Should be able to load CLI tasks directory");

        assert!(
            !tasks.is_empty(),
            "Should have loaded at least one CLI smoke task"
        );

        let cli_task_ids: Vec<&str> = tasks
            .iter()
            .filter(|t| t.id.starts_with("SMOKE-CLI-"))
            .map(|t| t.id.as_str())
            .collect();

        assert_eq!(
            cli_task_ids.len(),
            tasks.len(),
            "All loaded tasks should be SMOKE-CLI-* tasks"
        );
    }

    #[test]
    fn all_smoke_cli_tasks_have_valid_schema() {
        let loader = DefaultTaskLoader::new();
        let validator = DefaultTaskSchemaValidator::new();
        let tasks = loader
            .load_from_dir(&get_cli_tasks_dir())
            .expect("Should be able to load CLI tasks directory");

        for task in &tasks {
            let result = validator.validate(task);
            assert!(
                result.is_ok(),
                "Task {} should pass schema validation: {:?}",
                task.id,
                result
            );
        }
    }

    #[test]
    fn all_smoke_cli_tasks_have_cli_entry_mode() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_cli_tasks_dir())
            .expect("Should be able to load CLI tasks directory");

        for task in &tasks {
            assert_eq!(
                task.entry_mode,
                EntryMode::CLI,
                "Task {} should have CLI entry mode",
                task.id
            );
        }
    }

    #[test]
    fn all_smoke_cli_tasks_belong_to_smoke_category() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_cli_tasks_dir())
            .expect("Should be able to load CLI tasks directory");

        for task in &tasks {
            assert_eq!(
                task.category,
                TaskCategory::Smoke,
                "Task {} should belong to Smoke category",
                task.id
            );
        }
    }

    #[test]
    fn smoke_cli_tasks_load_count_matches_files() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_cli_tasks_dir())
            .expect("Should be able to load CLI tasks directory");

        let expected_count = 6;
        assert_eq!(
            tasks.len(),
            expected_count,
            "Expected {} SMOKE-CLI-* tasks, got {}",
            expected_count,
            tasks.len()
        );
    }
}

mod smoke_workspace_tests {
    use super::*;

    #[test]
    fn task_loader_can_load_all_smoke_ws_tasks() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_workspace_tasks_dir())
            .expect("Should be able to load workspace tasks directory");

        assert!(
            !tasks.is_empty(),
            "Should have loaded at least one workspace smoke task"
        );

        let ws_task_ids: Vec<&str> = tasks
            .iter()
            .filter(|t| t.id.starts_with("SMOKE-WS-"))
            .map(|t| t.id.as_str())
            .collect();

        assert_eq!(
            ws_task_ids.len(),
            tasks.len(),
            "All loaded tasks should be SMOKE-WS-* tasks"
        );
    }

    #[test]
    fn all_smoke_ws_tasks_have_valid_schema() {
        let loader = DefaultTaskLoader::new();
        let validator = DefaultTaskSchemaValidator::new();
        let tasks = loader
            .load_from_dir(&get_workspace_tasks_dir())
            .expect("Should be able to load workspace tasks directory");

        for task in &tasks {
            let result = validator.validate(task);
            assert!(
                result.is_ok(),
                "Task {} should pass schema validation: {:?}",
                task.id,
                result
            );
        }
    }

    #[test]
    fn all_smoke_ws_tasks_have_workspace_entry_mode() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_workspace_tasks_dir())
            .expect("Should be able to load workspace tasks directory");

        for task in &tasks {
            assert_eq!(
                task.entry_mode,
                EntryMode::Workspace,
                "Task {} should have Workspace entry mode",
                task.id
            );
        }
    }

    #[test]
    fn all_smoke_ws_tasks_belong_to_smoke_category() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_workspace_tasks_dir())
            .expect("Should be able to load workspace tasks directory");

        for task in &tasks {
            assert_eq!(
                task.category,
                TaskCategory::Smoke,
                "Task {} should belong to Smoke category",
                task.id
            );
        }
    }

    #[test]
    fn smoke_ws_tasks_load_count_matches_files() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_workspace_tasks_dir())
            .expect("Should be able to load workspace tasks directory");

        let expected_count = 5;
        assert_eq!(
            tasks.len(),
            expected_count,
            "Expected {} SMOKE-WS-* tasks, got {}",
            expected_count,
            tasks.len()
        );
    }
}

mod smoke_session_tests {
    use super::*;

    #[test]
    fn task_loader_can_load_all_smoke_session_tasks() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_session_tasks_dir())
            .expect("Should be able to load session tasks directory");

        assert!(
            !tasks.is_empty(),
            "Should have loaded at least one session smoke task"
        );

        let session_task_ids: Vec<&str> = tasks
            .iter()
            .filter(|t| t.id.starts_with("SMOKE-SESSION-"))
            .map(|t| t.id.as_str())
            .collect();

        assert_eq!(
            session_task_ids.len(),
            tasks.len(),
            "All loaded tasks should be SMOKE-SESSION-* tasks"
        );
    }

    #[test]
    fn all_smoke_session_tasks_have_valid_schema() {
        let loader = DefaultTaskLoader::new();
        let validator = DefaultTaskSchemaValidator::new();
        let tasks = loader
            .load_from_dir(&get_session_tasks_dir())
            .expect("Should be able to load session tasks directory");

        for task in &tasks {
            let result = validator.validate(task);
            assert!(
                result.is_ok(),
                "Task {} should pass schema validation: {:?}",
                task.id,
                result
            );
        }
    }

    #[test]
    fn all_smoke_session_tasks_have_session_entry_mode() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_session_tasks_dir())
            .expect("Should be able to load session tasks directory");

        for task in &tasks {
            assert_eq!(
                task.entry_mode,
                EntryMode::Session,
                "Task {} should have Session entry mode",
                task.id
            );
        }
    }

    #[test]
    fn all_smoke_session_tasks_belong_to_smoke_category() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_session_tasks_dir())
            .expect("Should be able to load session tasks directory");

        for task in &tasks {
            assert_eq!(
                task.category,
                TaskCategory::Smoke,
                "Task {} should belong to Smoke category",
                task.id
            );
        }
    }

    #[test]
    fn smoke_session_tasks_load_count_matches_files() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_session_tasks_dir())
            .expect("Should be able to load session tasks directory");

        let expected_count = 4;
        assert_eq!(
            tasks.len(),
            expected_count,
            "Expected {} SMOKE-SESSION-* tasks, got {}",
            expected_count,
            tasks.len()
        );
    }
}

mod smoke_permissions_tests {
    use super::*;

    #[test]
    fn task_loader_can_load_all_smoke_perm_tasks() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_permissions_tasks_dir())
            .expect("Should be able to load permissions tasks directory");

        assert!(
            !tasks.is_empty(),
            "Should have loaded at least one permissions smoke task"
        );

        let perm_task_ids: Vec<&str> = tasks
            .iter()
            .filter(|t| t.id.starts_with("SMOKE-PERM-"))
            .map(|t| t.id.as_str())
            .collect();

        assert_eq!(
            perm_task_ids.len(),
            tasks.len(),
            "All loaded tasks should be SMOKE-PERM-* tasks"
        );
    }

    #[test]
    fn all_smoke_perm_tasks_have_valid_schema() {
        let loader = DefaultTaskLoader::new();
        let validator = DefaultTaskSchemaValidator::new();
        let tasks = loader
            .load_from_dir(&get_permissions_tasks_dir())
            .expect("Should be able to load permissions tasks directory");

        for task in &tasks {
            let result = validator.validate(task);
            assert!(
                result.is_ok(),
                "Task {} should pass schema validation: {:?}",
                task.id,
                result
            );
        }
    }

    #[test]
    fn all_smoke_perm_tasks_have_permissions_entry_mode() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_permissions_tasks_dir())
            .expect("Should be able to load permissions tasks directory");

        for task in &tasks {
            assert_eq!(
                task.entry_mode,
                EntryMode::Permissions,
                "Task {} should have Permissions entry mode",
                task.id
            );
        }
    }

    #[test]
    fn all_smoke_perm_tasks_belong_to_smoke_category() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_permissions_tasks_dir())
            .expect("Should be able to load permissions tasks directory");

        for task in &tasks {
            assert_eq!(
                task.category,
                TaskCategory::Smoke,
                "Task {} should belong to Smoke category",
                task.id
            );
        }
    }

    #[test]
    fn smoke_perm_tasks_load_count_matches_files() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_permissions_tasks_dir())
            .expect("Should be able to load permissions tasks directory");

        let expected_count = 4;
        assert_eq!(
            tasks.len(),
            expected_count,
            "Expected {} SMOKE-PERM-* tasks, got {}",
            expected_count,
            tasks.len()
        );
    }
}

mod smoke_api_tests {
    use super::*;

    #[test]
    fn task_loader_can_load_all_smoke_api_tasks() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_api_tasks_dir())
            .expect("Should be able to load API tasks directory");

        assert!(
            !tasks.is_empty(),
            "Should have loaded at least one API smoke task"
        );

        let api_task_ids: Vec<&str> = tasks
            .iter()
            .filter(|t| t.id.starts_with("SMOKE-API-"))
            .map(|t| t.id.as_str())
            .collect();

        assert_eq!(
            api_task_ids.len(),
            tasks.len(),
            "All loaded tasks should be SMOKE-API-* tasks"
        );
    }

    #[test]
    fn all_smoke_api_tasks_have_valid_schema() {
        let loader = DefaultTaskLoader::new();
        let validator = DefaultTaskSchemaValidator::new();
        let tasks = loader
            .load_from_dir(&get_api_tasks_dir())
            .expect("Should be able to load API tasks directory");

        for task in &tasks {
            let result = validator.validate(task);
            assert!(
                result.is_ok(),
                "Task {} should pass schema validation: {:?}",
                task.id,
                result
            );
        }
    }

    #[test]
    fn all_smoke_api_tasks_have_api_entry_mode() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_api_tasks_dir())
            .expect("Should be able to load API tasks directory");

        for task in &tasks {
            assert_eq!(
                task.entry_mode,
                EntryMode::API,
                "Task {} should have API entry mode",
                task.id
            );
        }
    }

    #[test]
    fn all_smoke_api_tasks_belong_to_smoke_category() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_api_tasks_dir())
            .expect("Should be able to load API tasks directory");

        for task in &tasks {
            assert_eq!(
                task.category,
                TaskCategory::Smoke,
                "Task {} should belong to Smoke category",
                task.id
            );
        }
    }

    #[test]
    fn smoke_api_tasks_load_count_matches_files() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_api_tasks_dir())
            .expect("Should be able to load API tasks directory");

        let expected_count = 10;
        assert_eq!(
            tasks.len(),
            expected_count,
            "Expected {} SMOKE-API-* tasks, got {}",
            expected_count,
            tasks.len()
        );
    }
}

mod smoke_recovery_tests {
    use super::*;

    #[test]
    fn task_loader_can_load_all_smoke_recovery_tasks() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_recovery_tasks_dir())
            .expect("Should be able to load recovery tasks directory");

        assert!(
            !tasks.is_empty(),
            "Should have loaded at least one recovery smoke task"
        );

        let recovery_task_ids: Vec<&str> = tasks
            .iter()
            .filter(|t| t.id.starts_with("SMOKE-RECOVERY-"))
            .map(|t| t.id.as_str())
            .collect();

        assert_eq!(
            recovery_task_ids.len(),
            tasks.len(),
            "All loaded tasks should be SMOKE-RECOVERY-* tasks"
        );
    }

    #[test]
    fn all_smoke_recovery_tasks_have_valid_schema() {
        let loader = DefaultTaskLoader::new();
        let validator = DefaultTaskSchemaValidator::new();
        let tasks = loader
            .load_from_dir(&get_recovery_tasks_dir())
            .expect("Should be able to load recovery tasks directory");

        for task in &tasks {
            let result = validator.validate(task);
            assert!(
                result.is_ok(),
                "Task {} should pass schema validation: {:?}",
                task.id,
                result
            );
        }
    }

    #[test]
    fn all_smoke_recovery_tasks_have_recovery_entry_mode() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_recovery_tasks_dir())
            .expect("Should be able to load recovery tasks directory");

        for task in &tasks {
            assert_eq!(
                task.entry_mode,
                EntryMode::Recovery,
                "Task {} should have Recovery entry mode",
                task.id
            );
        }
    }

    #[test]
    fn all_smoke_recovery_tasks_belong_to_smoke_category() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_recovery_tasks_dir())
            .expect("Should be able to load recovery tasks directory");

        for task in &tasks {
            assert_eq!(
                task.category,
                TaskCategory::Smoke,
                "Task {} should belong to Smoke category",
                task.id
            );
        }
    }

    #[test]
    fn smoke_recovery_tasks_load_count_matches_files() {
        let loader = DefaultTaskLoader::new();
        let tasks = loader
            .load_from_dir(&get_recovery_tasks_dir())
            .expect("Should be able to load recovery tasks directory");

        let expected_count = 3;
        assert_eq!(
            tasks.len(),
            expected_count,
            "Expected {} SMOKE-RECOVERY-* tasks, got {}",
            expected_count,
            tasks.len()
        );
    }
}

mod smoke_task_loading_integration_tests {
    use super::*;

    #[test]
    fn task_loader_can_load_all_smoke_tasks() {
        let loader = DefaultTaskLoader::new();

        let cli_tasks = loader
            .load_from_dir(&get_cli_tasks_dir())
            .expect("Should be able to load CLI tasks");
        let ws_tasks = loader
            .load_from_dir(&get_workspace_tasks_dir())
            .expect("Should be able to load workspace tasks");
        let session_tasks = loader
            .load_from_dir(&get_session_tasks_dir())
            .expect("Should be able to load session tasks");
        let perm_tasks = loader
            .load_from_dir(&get_permissions_tasks_dir())
            .expect("Should be able to load permissions tasks");
        let api_tasks = loader
            .load_from_dir(&get_api_tasks_dir())
            .expect("Should be able to load API tasks");
        let recovery_tasks = loader
            .load_from_dir(&get_recovery_tasks_dir())
            .expect("Should be able to load recovery tasks");

        let total_smoke_tasks = cli_tasks.len()
            + ws_tasks.len()
            + session_tasks.len()
            + perm_tasks.len()
            + api_tasks.len()
            + recovery_tasks.len();

        assert_eq!(
            total_smoke_tasks, 32,
            "Expected 32 total smoke tasks (6 CLI + 5 WS + 4 SESSION + 4 PERM + 10 API + 3 RECOVERY), got {}",
            total_smoke_tasks
        );
    }

    #[test]
    fn all_smoke_tasks_pass_schema_validation() {
        let loader = DefaultTaskLoader::new();
        let validator = DefaultTaskSchemaValidator::new();

        let dirs = [
            get_cli_tasks_dir(),
            get_workspace_tasks_dir(),
            get_session_tasks_dir(),
            get_permissions_tasks_dir(),
            get_api_tasks_dir(),
            get_recovery_tasks_dir(),
        ];

        for dir in &dirs {
            let tasks = loader
                .load_from_dir(dir)
                .unwrap_or_else(|e| panic!("Failed to load tasks from {:?}: {}", dir, e));

            for task in &tasks {
                let result = validator.validate(task);
                assert!(
                    result.is_ok(),
                    "Task {} from {:?} should pass schema validation: {:?}",
                    task.id,
                    dir,
                    result
                );
            }
        }
    }
}
