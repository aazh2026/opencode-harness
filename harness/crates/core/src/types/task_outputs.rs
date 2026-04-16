#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TaskOutputs {
    pub stdout: String,
    pub stderr: String,
    pub files_created: Vec<String>,
    pub files_modified: Vec<String>,
}

impl TaskOutputs {
    pub fn new(
        stdout: impl Into<String>,
        stderr: impl Into<String>,
        files_created: Vec<String>,
        files_modified: Vec<String>,
    ) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: stderr.into(),
            files_created,
            files_modified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_outputs_creation() {
        let outputs = TaskOutputs::new(
            "test stdout".to_string(),
            "test stderr".to_string(),
            vec!["file1.txt".to_string(), "file2.rs".to_string()],
            vec!["file3.rs".to_string()],
        );

        assert_eq!(outputs.stdout, "test stdout");
        assert_eq!(outputs.stderr, "test stderr");
        assert_eq!(outputs.files_created.len(), 2);
        assert_eq!(outputs.files_modified.len(), 1);
    }

    #[test]
    fn test_task_outputs_serde_roundtrip() {
        let outputs = TaskOutputs::new(
            "stdout content",
            "stderr content",
            vec!["created1.txt".to_string()],
            vec!["modified1.rs".to_string(), "modified2.rs".to_string()],
        );

        let serialized = serde_json::to_string(&outputs).expect("serialization should succeed");
        let deserialized: TaskOutputs =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert_eq!(outputs, deserialized);
    }

    #[test]
    fn test_task_outputs_json_format() {
        let outputs = TaskOutputs::new("hello".to_string(), "".to_string(), vec![], vec![]);

        let json = serde_json::to_string(&outputs).unwrap();
        assert!(json.contains("\"stdout\":\"hello\""));
        assert!(json.contains("\"stderr\":\"\""));
        assert!(json.contains("\"files_created\":[]"));
        assert!(json.contains("\"files_modified\":[]"));
    }

    #[test]
    fn test_task_outputs_empty_fields() {
        let outputs = TaskOutputs::new(String::new(), String::new(), vec![], vec![]);

        assert!(outputs.stdout.is_empty());
        assert!(outputs.stderr.is_empty());
        assert!(outputs.files_created.is_empty());
        assert!(outputs.files_modified.is_empty());
    }

    #[test]
    fn test_task_outputs_fields_accessible() {
        let outputs = TaskOutputs::new(
            "out".to_string(),
            "err".to_string(),
            vec!["a.txt".to_string()],
            vec!["b.txt".to_string()],
        );

        let _: &str = &outputs.stdout;
        let _: &str = &outputs.stderr;
        let _: &Vec<String> = &outputs.files_created;
        let _: &Vec<String> = &outputs.files_modified;
    }
}
