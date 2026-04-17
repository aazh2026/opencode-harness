use crate::error::{ErrorType, Result};
use crate::types::allowed_variance::{WhitelistEntry, WhitelistScope};
use std::fs;
use std::path::{Path, PathBuf};

pub trait WhitelistLoader: Send + Sync {
    fn load(&self, whitelist_id: &str) -> Result<Option<WhitelistEntry>>;
    fn load_all(&self) -> Result<Vec<WhitelistEntry>>;
    fn load_active(&self) -> Result<Vec<WhitelistEntry>>;
    fn load_for_scope(&self, scope: &WhitelistScope) -> Result<Vec<WhitelistEntry>>;
    fn save(&self, entry: &WhitelistEntry) -> Result<()>;
    fn delete(&self, whitelist_id: &str) -> Result<()>;
    fn cleanup_expired(&self) -> Result<Vec<String>>;
}

#[derive(Debug, Clone)]
pub struct DefaultWhitelistLoader {
    base_path: PathBuf,
}

impl DefaultWhitelistLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn whitelist_path(&self, whitelist_id: &str) -> PathBuf {
        self.base_path.join(format!("{}.yaml", whitelist_id))
    }

    fn load_yaml_file(&self, path: &Path) -> Result<WhitelistEntry> {
        let content = fs::read_to_string(path).map_err(ErrorType::Io)?;
        serde_yaml::from_str(&content)
            .map_err(|e| ErrorType::Config(format!("Failed to parse whitelist YAML: {}", e)))
    }

    fn save_yaml_file(&self, path: &Path, entry: &WhitelistEntry) -> Result<()> {
        let content = serde_yaml::to_string(entry).map_err(|e| {
            ErrorType::Config(format!("Failed to serialize whitelist to YAML: {}", e))
        })?;
        fs::write(path, content).map_err(ErrorType::Io)?;
        Ok(())
    }

    fn all_yaml_files(&self) -> Result<Vec<PathBuf>> {
        if !self.base_path.is_dir() {
            return Ok(Vec::new());
        }

        let mut paths = Vec::new();
        for entry in fs::read_dir(&self.base_path).map_err(ErrorType::Io)? {
            let entry = entry.map_err(ErrorType::Io)?;
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        paths.push(file_path);
                    }
                }
            }
        }
        Ok(paths)
    }
}

impl Default for DefaultWhitelistLoader {
    fn default() -> Self {
        Self::new(PathBuf::from("harness/golden/whitelists"))
    }
}

impl WhitelistLoader for DefaultWhitelistLoader {
    fn load(&self, whitelist_id: &str) -> Result<Option<WhitelistEntry>> {
        let path = self.whitelist_path(whitelist_id);
        if !path.is_file() {
            return Ok(None);
        }
        match self.load_yaml_file(&path) {
            Ok(entry) => Ok(Some(entry)),
            Err(e) => Err(e),
        }
    }

    fn load_all(&self) -> Result<Vec<WhitelistEntry>> {
        let mut entries = Vec::new();
        for file_path in self.all_yaml_files()? {
            match self.load_yaml_file(&file_path) {
                Ok(entry) => entries.push(entry),
                Err(e) => return Err(e),
            }
        }
        entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(entries)
    }

    fn load_active(&self) -> Result<Vec<WhitelistEntry>> {
        let all_entries = self.load_all()?;
        Ok(all_entries
            .into_iter()
            .filter(|entry| !entry.is_expired())
            .collect())
    }

    fn load_for_scope(&self, scope: &WhitelistScope) -> Result<Vec<WhitelistEntry>> {
        let all_entries = self.load_all()?;
        Ok(all_entries
            .into_iter()
            .filter(|entry| entry.scope == *scope)
            .collect())
    }

    fn save(&self, entry: &WhitelistEntry) -> Result<()> {
        let path = self.whitelist_path(&entry.id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ErrorType::Io)?;
        }
        self.save_yaml_file(&path, entry)
    }

    fn delete(&self, whitelist_id: &str) -> Result<()> {
        let path = self.whitelist_path(whitelist_id);
        if !path.is_file() {
            return Err(ErrorType::Config(format!(
                "Whitelist not found: {}",
                whitelist_id
            )));
        }
        fs::remove_file(path).map_err(ErrorType::Io)?;
        Ok(())
    }

    fn cleanup_expired(&self) -> Result<Vec<String>> {
        let all_entries = self.load_all()?;
        let mut deleted_ids = Vec::new();

        for entry in all_entries {
            if entry.is_expired() {
                let path = self.whitelist_path(&entry.id);
                if path.is_file() {
                    fs::remove_file(&path).map_err(ErrorType::Io)?;
                    deleted_ids.push(entry.id);
                }
            }
        }

        Ok(deleted_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::allowed_variance::{AllowedVariance, TimingVariance};
    use chrono::Utc;
    use tempfile::TempDir;

    fn create_test_whitelist_entry(
        id: &str,
        scope: WhitelistScope,
        expires_at: Option<chrono::DateTime<Utc>>,
        created_at: chrono::DateTime<Utc>,
        updated_at: chrono::DateTime<Utc>,
    ) -> WhitelistEntry {
        WhitelistEntry::new(
            id.to_string(),
            scope,
            "Test reason".to_string(),
            "team-test".to_string(),
            expires_at,
            Some("https://github.com/example/repo/issues/123".to_string()),
            AllowedVariance::new(
                vec![0],
                Some(TimingVariance::new(Some(0), Some(1000))),
                vec![],
            ),
            created_at,
            updated_at,
        )
    }

    #[test]
    fn test_default_loader_creation() {
        let loader = DefaultWhitelistLoader::default();
        assert_eq!(loader.base_path, PathBuf::from("harness/golden/whitelists"));
    }

    #[test]
    fn test_loader_with_custom_base_path() {
        let loader = DefaultWhitelistLoader::new(PathBuf::from("/custom/path"));
        assert_eq!(loader.base_path, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_loader_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultWhitelistLoader>();
    }

    #[test]
    fn test_load_retrieves_whitelist_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let entry = create_test_whitelist_entry(
            "WL-001",
            WhitelistScope::Task("TASK-001".to_string()),
            Some(future),
            now,
            now,
        );
        loader.save(&entry).unwrap();

        let loaded = loader.load("WL-001").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, "WL-001");
        assert_eq!(loaded.scope, WhitelistScope::Task("TASK-001".to_string()));
        assert_eq!(loaded.owner, "team-test");
    }

    #[test]
    fn test_load_returns_none_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let loaded = loader.load("nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_load_all_returns_all_whitelist_entries() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);

        let entry1 = create_test_whitelist_entry(
            "WL-001",
            WhitelistScope::Task("TASK-001".to_string()),
            Some(future),
            now,
            now,
        );
        let entry2 = create_test_whitelist_entry(
            "WL-002",
            WhitelistScope::Category("timing".to_string()),
            Some(future),
            now,
            now,
        );
        let entry3 =
            create_test_whitelist_entry("WL-003", WhitelistScope::Global, Some(future), now, now);

        loader.save(&entry1).unwrap();
        loader.save(&entry2).unwrap();
        loader.save(&entry3).unwrap();

        let all_entries = loader.load_all().unwrap();
        assert_eq!(all_entries.len(), 3);
    }

    #[test]
    fn test_load_all_returns_empty_for_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let all_entries = loader.load_all().unwrap();
        assert!(all_entries.is_empty());
    }

    #[test]
    fn test_load_active_returns_only_non_expired_entries() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let past = now - chrono::Duration::days(1);

        let active_entry = create_test_whitelist_entry(
            "WL-ACTIVE",
            WhitelistScope::Task("TASK-001".to_string()),
            Some(future),
            now,
            now,
        );
        let expired_entry = create_test_whitelist_entry(
            "WL-EXPIRED",
            WhitelistScope::Task("TASK-002".to_string()),
            Some(past),
            now,
            now,
        );

        loader.save(&active_entry).unwrap();
        loader.save(&expired_entry).unwrap();

        let active_entries = loader.load_active().unwrap();
        assert_eq!(active_entries.len(), 1);
        assert_eq!(active_entries[0].id, "WL-ACTIVE");
    }

    #[test]
    fn test_load_active_returns_empty_when_all_expired() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let past = now - chrono::Duration::days(1);

        let expired1 = create_test_whitelist_entry(
            "WL-EXPIRED1",
            WhitelistScope::Global,
            Some(past),
            now,
            now,
        );
        let expired2 = create_test_whitelist_entry(
            "WL-EXPIRED2",
            WhitelistScope::Global,
            Some(past),
            now,
            now,
        );

        loader.save(&expired1).unwrap();
        loader.save(&expired2).unwrap();

        let active_entries = loader.load_active().unwrap();
        assert!(active_entries.is_empty());
    }

    #[test]
    fn test_load_for_scope_filters_by_task_scope() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);

        let entry1 = create_test_whitelist_entry(
            "WL-001",
            WhitelistScope::Task("TASK-001".to_string()),
            Some(future),
            now,
            now,
        );
        let entry2 = create_test_whitelist_entry(
            "WL-002",
            WhitelistScope::Task("TASK-002".to_string()),
            Some(future),
            now,
            now,
        );
        let entry3 = create_test_whitelist_entry(
            "WL-003",
            WhitelistScope::Category("timing".to_string()),
            Some(future),
            now,
            now,
        );

        loader.save(&entry1).unwrap();
        loader.save(&entry2).unwrap();
        loader.save(&entry3).unwrap();

        let task_entries = loader
            .load_for_scope(&WhitelistScope::Task("TASK-001".to_string()))
            .unwrap();
        assert_eq!(task_entries.len(), 1);
        assert_eq!(task_entries[0].id, "WL-001");
    }

    #[test]
    fn test_load_for_scope_filters_by_category_scope() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);

        let entry1 = create_test_whitelist_entry(
            "WL-001",
            WhitelistScope::Category("timing".to_string()),
            Some(future),
            now,
            now,
        );
        let entry2 = create_test_whitelist_entry(
            "WL-002",
            WhitelistScope::Category("output".to_string()),
            Some(future),
            now,
            now,
        );
        let entry3 =
            create_test_whitelist_entry("WL-003", WhitelistScope::Global, Some(future), now, now);

        loader.save(&entry1).unwrap();
        loader.save(&entry2).unwrap();
        loader.save(&entry3).unwrap();

        let category_entries = loader
            .load_for_scope(&WhitelistScope::Category("timing".to_string()))
            .unwrap();
        assert_eq!(category_entries.len(), 1);
        assert_eq!(category_entries[0].id, "WL-001");
    }

    #[test]
    fn test_load_for_scope_returns_global_entries() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);

        let entry = create_test_whitelist_entry(
            "WL-GLOBAL",
            WhitelistScope::Global,
            Some(future),
            now,
            now,
        );

        loader.save(&entry).unwrap();

        let global_entries = loader.load_for_scope(&WhitelistScope::Global).unwrap();
        assert_eq!(global_entries.len(), 1);
        assert_eq!(global_entries[0].id, "WL-GLOBAL");
    }

    #[test]
    fn test_save_persists_to_correct_yaml_path() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let entry = create_test_whitelist_entry(
            "WL-001",
            WhitelistScope::Task("TASK-001".to_string()),
            Some(future),
            now,
            now,
        );
        loader.save(&entry).unwrap();

        let expected_path = temp_dir.path().join("WL-001.yaml");
        assert!(expected_path.is_file());

        let content = fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains("WL-001"));
        assert!(content.contains("TASK-001"));
        assert!(content.contains("team-test"));
    }

    #[test]
    fn test_save_load_roundtrip_preserves_all_data() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let original = create_test_whitelist_entry(
            "WL-001",
            WhitelistScope::Task("TASK-001".to_string()),
            Some(future),
            now,
            now,
        );

        loader.save(&original).unwrap();

        let loaded = loader.load("WL-001").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();

        assert_eq!(loaded.id, original.id);
        assert_eq!(loaded.scope, original.scope);
        assert_eq!(loaded.reason, original.reason);
        assert_eq!(loaded.owner, original.owner);
        assert_eq!(loaded.expires_at, original.expires_at);
        assert_eq!(loaded.linked_issue, original.linked_issue);
        assert_eq!(
            loaded.allowed_variance.exit_code,
            original.allowed_variance.exit_code
        );
        assert_eq!(loaded.created_at, original.created_at);
        assert_eq!(loaded.updated_at, original.updated_at);
    }

    #[test]
    fn test_delete_removes_whitelist() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let entry =
            create_test_whitelist_entry("WL-001", WhitelistScope::Global, Some(future), now, now);
        loader.save(&entry).unwrap();

        let path = temp_dir.path().join("WL-001.yaml");
        assert!(path.is_file());

        loader.delete("WL-001").unwrap();
        assert!(!path.is_file());
    }

    #[test]
    fn test_delete_returns_error_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let result = loader.delete("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup_expired_removes_expired_entries() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let past = now - chrono::Duration::days(1);

        let active_entry = create_test_whitelist_entry(
            "WL-ACTIVE",
            WhitelistScope::Global,
            Some(future),
            now,
            now,
        );
        let expired_entry =
            create_test_whitelist_entry("WL-EXPIRED", WhitelistScope::Global, Some(past), now, now);

        loader.save(&active_entry).unwrap();
        loader.save(&expired_entry).unwrap();

        let deleted_ids = loader.cleanup_expired().unwrap();
        assert_eq!(deleted_ids.len(), 1);
        assert_eq!(deleted_ids[0], "WL-EXPIRED");

        let remaining = loader.load_all().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "WL-ACTIVE");
    }

    #[test]
    fn test_cleanup_expired_returns_empty_when_no_expired() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);

        let entry = create_test_whitelist_entry(
            "WL-ACTIVE",
            WhitelistScope::Global,
            Some(future),
            now,
            now,
        );

        loader.save(&entry).unwrap();

        let deleted_ids = loader.cleanup_expired().unwrap();
        assert!(deleted_ids.is_empty());

        let remaining = loader.load_all().unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn test_whitelist_loader_trait_object() {
        let temp_dir = TempDir::new().unwrap();
        let loader: Box<dyn WhitelistLoader> =
            Box::new(DefaultWhitelistLoader::new(temp_dir.path().to_path_buf()));

        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let entry =
            create_test_whitelist_entry("WL-001", WhitelistScope::Global, Some(future), now, now);
        loader.save(&entry).unwrap();

        let loaded = loader.load("WL-001").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, "WL-001");
    }
}
