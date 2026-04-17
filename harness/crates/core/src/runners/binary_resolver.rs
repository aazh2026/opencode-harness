use crate::error::{ErrorType, Result};
use std::env;
use std::path::PathBuf;

pub struct BinaryResolver {
    search_paths: Vec<PathBuf>,
}

impl BinaryResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            search_paths: Vec::new(),
        };
        resolver.add_default_paths();
        resolver
    }

    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    fn add_default_paths(&mut self) {
        if let Ok(path_var) = env::var("PATH") {
            for p in path_var.split(':') {
                self.search_paths.push(PathBuf::from(p));
            }
        }

        #[cfg(target_os = "macos")]
        {
            self.search_paths.push(PathBuf::from("/usr/local/bin"));
            self.search_paths.push(PathBuf::from("/opt/homebrew/bin"));
        }

        #[cfg(target_os = "linux")]
        {
            self.search_paths.push(PathBuf::from("/usr/local/bin"));
            self.search_paths.push(PathBuf::from("/usr/bin"));
        }

        #[cfg(target_os = "windows")]
        {
            self.search_paths.push(PathBuf::from("C:\\Program Files"));
            self.search_paths
                .push(PathBuf::from("C:\\Program Files (x86)"));
        }
    }

    fn check_env(&self, var: &str) -> Option<PathBuf> {
        env::var(var).ok().map(PathBuf::from).filter(|p| p.exists())
    }

    fn find_in_paths(&self, name: &str) -> Option<PathBuf> {
        for dir in &self.search_paths {
            let candidate = dir.join(name);
            if candidate.exists() {
                let metadata = std::fs::metadata(&candidate).ok()?;
                if metadata.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    pub fn resolve(&self, name: &str) -> Result<PathBuf> {
        if let Some(path) = self.find_in_paths(name) {
            return Ok(path);
        }
        let search_paths = self
            .search_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Err(ErrorType::Runner(format!(
            "Could not find binary '{}' in PATH or default locations. Searched in: [{}]",
            name, search_paths
        )))
    }

    pub fn resolve_opencode(&self) -> Result<PathBuf> {
        self.resolve_opencode_with_override(None)
    }

    pub fn resolve_opencode_with_override(&self, binary_path: Option<&PathBuf>) -> Result<PathBuf> {
        if let Some(path) = binary_path {
            if path.exists() {
                return Ok(path.clone());
            }
            return Err(ErrorType::Runner(format!(
                "Provided binary_path '{}' does not exist",
                path.display()
            )));
        }
        if let Some(path) = self.check_env("OPENCODE_PATH") {
            return Ok(path);
        }
        self.resolve("opencode")
    }

    pub fn resolve_opencode_rs(&self) -> Result<PathBuf> {
        self.resolve_opencode_rs_with_override(None)
    }

    pub fn resolve_opencode_rs_with_override(
        &self,
        binary_path: Option<&PathBuf>,
    ) -> Result<PathBuf> {
        if let Some(path) = binary_path {
            if path.exists() {
                return Ok(path.clone());
            }
            return Err(ErrorType::Runner(format!(
                "Provided binary_path '{}' does not exist",
                path.display()
            )));
        }
        if let Some(path) = self.check_env("OPENCODE_RS_PATH") {
            return Ok(path);
        }
        self.resolve("opencode-rs")
    }
}

impl Default for BinaryResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_binary_resolver_creation() {
        let resolver = BinaryResolver::new();
        assert!(!resolver.search_paths.is_empty());
    }

    #[test]
    fn test_binary_resolver_add_search_path() {
        let mut resolver = BinaryResolver::new();
        resolver.add_search_path(PathBuf::from("/custom/path"));
        assert!(resolver
            .search_paths
            .contains(&PathBuf::from("/custom/path")));
    }

    #[test]
    fn test_resolve_echo_exists() {
        let resolver = BinaryResolver::new();
        #[cfg(target_os = "macos")]
        let result = resolver.resolve("echo");
        #[cfg(target_os = "linux")]
        let result = resolver.resolve("echo");

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            assert!(result.is_ok());
            let path = result.unwrap();
            assert!(path.exists());
        }
    }

    #[test]
    fn test_resolve_nonexistent_binary() {
        let resolver = BinaryResolver::new();
        let result = resolver.resolve("nonexistent_binary_xyz_123");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_opencode_path_env() {
        let resolver = BinaryResolver::new();
        env::set_var("OPENCODE_PATH", "/usr/bin/false");
        let result = resolver.resolve_opencode();
        env::remove_var("OPENCODE_PATH");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/usr/bin/false"));
    }

    #[test]
    fn test_binary_path_checked_first_in_resolve_opencode_rs_with_override() {
        let temp_dir = TempDir::new().unwrap();
        let custom_path = temp_dir.path().join("opencode-rs");

        #[cfg(unix)]
        std::fs::write(&custom_path, "").unwrap();
        #[cfg(windows)]
        std::fs::write(&custom_path, "").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&custom_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let resolver = BinaryResolver::new();
        let result = resolver.resolve_opencode_rs_with_override(Some(&custom_path));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), custom_path);
    }

    #[test]
    fn test_binary_path_nonexistent_returns_error() {
        let resolver = BinaryResolver::new();
        let nonexistent = PathBuf::from("/nonexistent/path/to/opencode-rs");
        let result = resolver.resolve_opencode_rs_with_override(Some(&nonexistent));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("does not exist"));
    }

    #[test]
    fn test_binary_path_takes_precedence_over_env_var() {
        let temp_dir = TempDir::new().unwrap();
        let custom_path = temp_dir.path().join("opencode-rs");

        #[cfg(unix)]
        std::fs::write(&custom_path, "").unwrap();
        #[cfg(windows)]
        std::fs::write(&custom_path, "").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&custom_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let resolver = BinaryResolver::new();
        env::set_var("OPENCODE_RS_PATH", "/usr/bin/false");
        let result = resolver.resolve_opencode_rs_with_override(Some(&custom_path));
        env::remove_var("OPENCODE_RS_PATH");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), custom_path);
    }
}
