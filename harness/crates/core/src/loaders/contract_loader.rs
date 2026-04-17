use crate::error::{ErrorType, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub contract_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
}

impl Contract {
    pub fn new(contract_id: String, name: String, version: String, description: String) -> Self {
        Self {
            contract_id,
            name,
            version,
            description,
        }
    }
}

pub trait ContractLoader: Send + Sync {
    fn load_from_dir(&self, path: &Path) -> Result<Vec<Contract>>;
    fn load_single(&self, path: &Path) -> Result<Contract>;
}

pub struct DefaultContractLoader;

impl DefaultContractLoader {
    pub fn new() -> Self {
        Self
    }

    fn load_yaml_file(&self, path: &Path) -> Result<Contract> {
        let content = fs::read_to_string(path).map_err(ErrorType::Io)?;
        serde_yaml::from_str(&content)
            .map_err(|e| ErrorType::Config(format!("Failed to parse contract YAML: {}", e)))
    }
}

impl Default for DefaultContractLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractLoader for DefaultContractLoader {
    fn load_from_dir(&self, path: &Path) -> Result<Vec<Contract>> {
        if !path.is_dir() {
            return Err(ErrorType::Config(format!(
                "Path is not a directory: {}",
                path.display()
            )));
        }

        let mut contracts = Vec::new();
        for entry in fs::read_dir(path).map_err(ErrorType::Io)? {
            let entry = entry.map_err(ErrorType::Io)?;
            let file_path = entry.path();

            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        match self.load_yaml_file(&file_path) {
                            Ok(contract) => contracts.push(contract),
                            Err(e) => {
                                return Err(ErrorType::Config(format!(
                                    "Failed to load contract from {}: {}",
                                    file_path.display(),
                                    e
                                )));
                            }
                        }
                    }
                }
            }
        }

        Ok(contracts)
    }

    fn load_single(&self, path: &Path) -> Result<Contract> {
        if !path.is_file() {
            return Err(ErrorType::Config(format!(
                "Path is not a file: {}",
                path.display()
            )));
        }

        self.load_yaml_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_valid_contract_yaml() -> &'static str {
        r#"
contract_id: TEST-CONTRACT-001
name: Test Contract
version: 1.0.0
description: A test contract for validation
"#
    }

    fn create_second_contract_yaml() -> &'static str {
        r#"
contract_id: TEST-CONTRACT-002
name: Second Test Contract
version: 1.0.0
description: Another test contract
"#
    }

    #[test]
    fn test_default_loader_creation() {
        let loader = DefaultContractLoader::new();
        drop(loader);
    }

    #[test]
    fn test_loader_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultContractLoader>();
    }

    #[test]
    fn test_load_single_valid_yaml() {
        let loader = DefaultContractLoader::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("contract.yaml");

        std::fs::write(&file_path, create_valid_contract_yaml()).unwrap();

        let contract = loader.load_single(&file_path).unwrap();
        assert_eq!(contract.contract_id, "TEST-CONTRACT-001");
        assert_eq!(contract.name, "Test Contract");
    }

    #[test]
    fn test_load_single_invalid_yaml() {
        let loader = DefaultContractLoader::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.yaml");

        std::fs::write(&file_path, "invalid: yaml: content: [").unwrap();

        assert!(loader.load_single(&file_path).is_err());
    }

    #[test]
    fn test_load_single_nonexistent_file() {
        let loader = DefaultContractLoader::new();
        let result = loader.load_single(Path::new("/nonexistent/path/contract.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_single_directory_error() {
        let loader = DefaultContractLoader::new();
        let temp_dir = TempDir::new().unwrap();

        let result = loader.load_single(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_dir_multiple_contracts() {
        let loader = DefaultContractLoader::new();
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(
            temp_dir.path().join("contract1.yaml"),
            create_valid_contract_yaml(),
        )
        .unwrap();
        std::fs::write(
            temp_dir.path().join("contract2.yaml"),
            create_second_contract_yaml(),
        )
        .unwrap();

        let contracts = loader.load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(contracts.len(), 2);

        let ids: Vec<&str> = contracts.iter().map(|c| c.contract_id.as_str()).collect();
        assert!(ids.contains(&"TEST-CONTRACT-001"));
        assert!(ids.contains(&"TEST-CONTRACT-002"));
    }

    #[test]
    fn test_load_from_dir_empty_directory() {
        let loader = DefaultContractLoader::new();
        let temp_dir = TempDir::new().unwrap();

        let contracts = loader.load_from_dir(temp_dir.path()).unwrap();
        assert!(contracts.is_empty());
    }

    #[test]
    fn test_load_from_dir_with_subdirectories() {
        let loader = DefaultContractLoader::new();
        let temp_dir = TempDir::new().unwrap();

        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::write(
            temp_dir.path().join("contract.yaml"),
            create_valid_contract_yaml(),
        )
        .unwrap();

        let contracts = loader.load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].contract_id, "TEST-CONTRACT-001");
    }

    #[test]
    fn test_load_from_dir_skips_non_yaml_files() {
        let loader = DefaultContractLoader::new();
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(temp_dir.path().join("readme.txt"), "not a contract").unwrap();
        std::fs::write(temp_dir.path().join("data.json"), "{\"key\": \"value\"}").unwrap();

        let contracts = loader.load_from_dir(temp_dir.path()).unwrap();
        assert!(contracts.is_empty());
    }

    #[test]
    fn test_contract_struct_creation() {
        let contract = Contract::new(
            "CONTRACT-001".to_string(),
            "Test".to_string(),
            "1.0.0".to_string(),
            "Description".to_string(),
        );
        assert_eq!(contract.contract_id, "CONTRACT-001");
        assert_eq!(contract.name, "Test");
        assert_eq!(contract.version, "1.0.0");
        assert_eq!(contract.description, "Description");
    }
}
