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
        Err(ErrorType::Runner(format!(
            "Could not find binary '{}' in PATH or default locations",
            name
        )))
    }

    pub fn resolve_opencode(&self) -> Result<PathBuf> {
        if let Some(path) = self.check_env("OPENCODE_PATH") {
            return Ok(path);
        }
        self.resolve("opencode")
    }

    pub fn resolve_opencode_rs(&self) -> Result<PathBuf> {
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
}
