use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssertionType {
    ExitCodeEquals(u32),
    StdoutContains(String),
    StderrContains(String),
    FileChanged(String),
    NoExtraFilesChanged,
    PermissionPromptSeen(String),
}

impl Serialize for AssertionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct ExitCodeEquals {
            #[serde(rename = "type")]
            type_: &'static str,
            value: u32,
        }
        #[derive(Serialize)]
        struct StringValue {
            #[serde(rename = "type")]
            type_: &'static str,
            value: String,
        }
        #[derive(Serialize)]
        struct NoExtraFilesChanged {
            #[serde(rename = "type")]
            type_: &'static str,
        }

        match self {
            AssertionType::ExitCodeEquals(code) => ExitCodeEquals {
                type_: "exit_code_equals",
                value: *code,
            }
            .serialize(serializer),
            AssertionType::StdoutContains(s) => StringValue {
                type_: "stdout_contains",
                value: s.clone(),
            }
            .serialize(serializer),
            AssertionType::StderrContains(s) => StringValue {
                type_: "stderr_contains",
                value: s.clone(),
            }
            .serialize(serializer),
            AssertionType::FileChanged(s) => StringValue {
                type_: "file_changed",
                value: s.clone(),
            }
            .serialize(serializer),
            AssertionType::NoExtraFilesChanged => NoExtraFilesChanged {
                type_: "no_extra_files_changed",
            }
            .serialize(serializer),
            AssertionType::PermissionPromptSeen(s) => StringValue {
                type_: "permission_prompt_seen",
                value: s.clone(),
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for AssertionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AssertionTypeHelper {
            #[serde(rename = "type")]
            type_: String,
            value: Option<serde_json::value::Value>,
        }

        let helper = AssertionTypeHelper::deserialize(deserializer)?;

        match helper.type_.as_str() {
            "exit_code_equals" => {
                let code = helper
                    .value
                    .ok_or_else(|| serde::de::Error::missing_field("value"))?;
                let code = code.as_u64().ok_or_else(|| {
                    serde::de::Error::custom("value must be a non-negative integer")
                })? as u32;
                Ok(AssertionType::ExitCodeEquals(code))
            }
            "stdout_contains" => {
                let s = helper
                    .value
                    .ok_or_else(|| serde::de::Error::missing_field("value"))?;
                let s = s
                    .as_str()
                    .ok_or_else(|| serde::de::Error::custom("value must be a string"))?;
                Ok(AssertionType::StdoutContains(s.to_string()))
            }
            "stderr_contains" => {
                let s = helper
                    .value
                    .ok_or_else(|| serde::de::Error::missing_field("value"))?;
                let s = s
                    .as_str()
                    .ok_or_else(|| serde::de::Error::custom("value must be a string"))?;
                Ok(AssertionType::StderrContains(s.to_string()))
            }
            "file_changed" => {
                let s = helper
                    .value
                    .ok_or_else(|| serde::de::Error::missing_field("value"))?;
                let s = s
                    .as_str()
                    .ok_or_else(|| serde::de::Error::custom("value must be a string"))?;
                Ok(AssertionType::FileChanged(s.to_string()))
            }
            "no_extra_files_changed" => Ok(AssertionType::NoExtraFilesChanged),
            "permission_prompt_seen" => {
                let s = helper
                    .value
                    .ok_or_else(|| serde::de::Error::missing_field("value"))?;
                let s = s
                    .as_str()
                    .ok_or_else(|| serde::de::Error::custom("value must be a string"))?;
                Ok(AssertionType::PermissionPromptSeen(s.to_string()))
            }
            _ => Err(serde::de::Error::custom(format!(
                "unknown assertion type: {}",
                helper.type_
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assertion_type_exit_code_equals() {
        let assertion = AssertionType::ExitCodeEquals(0);
        let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
        assert_eq!(serialized, "{\"type\":\"exit_code_equals\",\"value\":0}");

        let deserialized: AssertionType =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(assertion, deserialized);
    }

    #[test]
    fn test_assertion_type_stdout_contains() {
        let assertion = AssertionType::StdoutContains("Usage: opencode".to_string());
        let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
        assert_eq!(
            serialized,
            "{\"type\":\"stdout_contains\",\"value\":\"Usage: opencode\"}"
        );

        let deserialized: AssertionType =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(assertion, deserialized);
    }

    #[test]
    fn test_assertion_type_stderr_contains() {
        let assertion = AssertionType::StderrContains("Error: permission denied".to_string());
        let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
        assert_eq!(
            serialized,
            "{\"type\":\"stderr_contains\",\"value\":\"Error: permission denied\"}"
        );

        let deserialized: AssertionType =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(assertion, deserialized);
    }

    #[test]
    fn test_assertion_type_file_changed() {
        let assertion = AssertionType::FileChanged("src/main.rs".to_string());
        let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
        assert_eq!(
            serialized,
            "{\"type\":\"file_changed\",\"value\":\"src/main.rs\"}"
        );

        let deserialized: AssertionType =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(assertion, deserialized);
    }

    #[test]
    fn test_assertion_type_no_extra_files_changed() {
        let assertion = AssertionType::NoExtraFilesChanged;
        let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
        assert_eq!(serialized, "{\"type\":\"no_extra_files_changed\"}");

        let deserialized: AssertionType =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(assertion, deserialized);
    }

    #[test]
    fn test_assertion_type_permission_prompt_seen() {
        let assertion = AssertionType::PermissionPromptSeen("Allow access?".to_string());
        let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
        assert_eq!(
            serialized,
            "{\"type\":\"permission_prompt_seen\",\"value\":\"Allow access?\"}"
        );

        let deserialized: AssertionType =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(assertion, deserialized);
    }

    #[test]
    fn test_assertion_type_serde_roundtrip() {
        let variants = [
            AssertionType::ExitCodeEquals(0),
            AssertionType::ExitCodeEquals(1),
            AssertionType::StdoutContains("Hello".to_string()),
            AssertionType::StderrContains("Error".to_string()),
            AssertionType::FileChanged("path/to/file".to_string()),
            AssertionType::NoExtraFilesChanged,
            AssertionType::PermissionPromptSeen("?".to_string()),
        ];

        for original in variants {
            let serialized =
                serde_json::to_string(&original).expect("serialization should succeed");
            let deserialized: AssertionType =
                serde_json::from_str(&serialized).expect("deserialization should succeed");
            assert_eq!(
                original, deserialized,
                "roundtrip should preserve the value"
            );
        }
    }

    #[test]
    fn test_assertion_type_json_format() {
        assert_eq!(
            serde_json::to_string(&AssertionType::ExitCodeEquals(0)).unwrap(),
            "{\"type\":\"exit_code_equals\",\"value\":0}"
        );
        assert_eq!(
            serde_json::to_string(&AssertionType::NoExtraFilesChanged).unwrap(),
            "{\"type\":\"no_extra_files_changed\"}"
        );
    }
}
