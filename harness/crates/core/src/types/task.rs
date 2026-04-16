use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub category: TaskCategory,
    pub fixture_project: String,
    pub description: String,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskCategory {
    Core,
    Schema,
    Integration,
    Regression,
    Smoke,
    Performance,
    Security,
}

impl Task {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        category: TaskCategory,
        fixture_project: impl Into<String>,
        description: impl Into<String>,
        expected_outcome: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category,
            fixture_project: fixture_project.into(),
            description: description.into(),
            expected_outcome: expected_outcome.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "P2-001",
            "Define Task Schema",
            TaskCategory::Schema,
            "fixtures/projects/example",
            "Create harness/tasks/schema.yaml defining Task struct",
            "Schema validates Task definitions correctly",
        );

        assert_eq!(task.id, "P2-001");
        assert_eq!(task.title, "Define Task Schema");
        assert_eq!(task.category, TaskCategory::Schema);
        assert_eq!(task.fixture_project, "fixtures/projects/example");
    }

    #[test]
    fn test_task_serde_yaml() {
        let task = Task::new(
            "P2-001",
            "Define Task Schema",
            TaskCategory::Schema,
            "fixtures/projects/example",
            "Create harness/tasks/schema.yaml defining Task struct",
            "Schema validates Task definitions correctly",
        );

        let yaml = serde_yaml::to_string(&task).unwrap();
        assert!(yaml.contains("id: P2-001"));
        assert!(yaml.contains("title: Define Task Schema"));
        assert!(yaml.contains("category: schema"));

        let deserialized: Task = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized, task);
    }

    #[test]
    fn test_task_serde_json() {
        let task = Task::new(
            "P2-001",
            "Define Task Schema",
            TaskCategory::Schema,
            "fixtures/projects/example",
            "Create harness/tasks/schema.yaml defining Task struct",
            "Schema validates Task definitions correctly",
        );

        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("\"id\":\"P2-001\""));
        assert!(json.contains("\"title\":\"Define Task Schema\""));
        assert!(json.contains("\"category\":\"schema\""));

        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, task);
    }

    #[test]
    fn test_task_category_variants() {
        let categories = vec![
            (TaskCategory::Core, "core"),
            (TaskCategory::Schema, "schema"),
            (TaskCategory::Integration, "integration"),
            (TaskCategory::Regression, "regression"),
            (TaskCategory::Smoke, "smoke"),
            (TaskCategory::Performance, "performance"),
            (TaskCategory::Security, "security"),
        ];

        for (category, expected_str) in categories {
            let json = serde_json::to_string(&category).unwrap();
            assert_eq!(json, format!("\"{}\"", expected_str));
        }
    }
}
