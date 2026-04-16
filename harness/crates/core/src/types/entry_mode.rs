#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EntryMode {
    CLI,
    API,
    Session,
    Permissions,
    Web,
    Workspace,
    Recovery,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_mode_variants_exist() {
        let _ = EntryMode::CLI;
        let _ = EntryMode::API;
        let _ = EntryMode::Session;
        let _ = EntryMode::Permissions;
        let _ = EntryMode::Web;
        let _ = EntryMode::Workspace;
        let _ = EntryMode::Recovery;
    }

    #[test]
    fn test_entry_mode_serde_roundtrip() {
        let variants = [
            EntryMode::CLI,
            EntryMode::API,
            EntryMode::Session,
            EntryMode::Permissions,
            EntryMode::Web,
            EntryMode::Workspace,
            EntryMode::Recovery,
        ];

        for original in variants {
            let serialized =
                serde_json::to_string(&original).expect("serialization should succeed");
            let deserialized: EntryMode =
                serde_json::from_str(&serialized).expect("deserialization should succeed");
            assert_eq!(
                original, deserialized,
                "roundtrip should preserve the value"
            );
        }
    }

    #[test]
    fn test_entry_mode_json_format() {
        assert_eq!(serde_json::to_string(&EntryMode::CLI).unwrap(), "\"CLI\"");
        assert_eq!(serde_json::to_string(&EntryMode::API).unwrap(), "\"API\"");
        assert_eq!(
            serde_json::to_string(&EntryMode::Session).unwrap(),
            "\"Session\""
        );
        assert_eq!(
            serde_json::to_string(&EntryMode::Permissions).unwrap(),
            "\"Permissions\""
        );
        assert_eq!(serde_json::to_string(&EntryMode::Web).unwrap(), "\"Web\"");
        assert_eq!(
            serde_json::to_string(&EntryMode::Workspace).unwrap(),
            "\"Workspace\""
        );
        assert_eq!(
            serde_json::to_string(&EntryMode::Recovery).unwrap(),
            "\"Recovery\""
        );
    }
}
