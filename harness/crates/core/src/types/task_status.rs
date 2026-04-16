#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    ManualCheck,
    Blocked,
    Skipped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_serde_roundtrip() {
        let variants = [
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Done,
            TaskStatus::ManualCheck,
            TaskStatus::Blocked,
            TaskStatus::Skipped,
        ];

        for original in variants {
            let serialized =
                serde_json::to_string(&original).expect("serialization should succeed");
            let deserialized: TaskStatus =
                serde_json::from_str(&serialized).expect("deserialization should succeed");
            assert_eq!(
                original, deserialized,
                "roundtrip should preserve the value"
            );
        }
    }

    #[test]
    fn test_task_status_json_format() {
        assert_eq!(
            serde_json::to_string(&TaskStatus::Todo).unwrap(),
            "\"Todo\""
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::InProgress).unwrap(),
            "\"InProgress\""
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::Done).unwrap(),
            "\"Done\""
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::ManualCheck).unwrap(),
            "\"ManualCheck\""
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::Blocked).unwrap(),
            "\"Blocked\""
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::Skipped).unwrap(),
            "\"Skipped\""
        );
    }
}
