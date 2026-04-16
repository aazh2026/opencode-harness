use std::env;
use std::process::Command;

pub trait EnvironmentProbe: Send + Sync {
    fn check_binary(&self, name: &str) -> bool;
    fn get_info(&self) -> EnvironmentInfo;
}

#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub os: String,
    pub arch: String,
    pub rustc_version: Option<String>,
    pub available_binaries: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DefaultEnvironmentProbe;

impl EnvironmentProbe for DefaultEnvironmentProbe {
    fn check_binary(&self, name: &str) -> bool {
        if cfg!(target_os = "windows") {
            Command::new("where")
                .arg(name)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        } else {
            Command::new("which")
                .arg(name)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
    }

    fn get_info(&self) -> EnvironmentInfo {
        let os = env::consts::OS.to_string();
        let arch = env::consts::ARCH.to_string();
        let rustc_version = Self::get_rustc_version();

        let common_binaries = [
            "cargo", "rustc", "rustup", "git", "npm", "node", "python", "python3",
        ];
        let available_binaries: Vec<String> = common_binaries
            .iter()
            .filter(|&bin| self.check_binary(bin))
            .copied()
            .map(String::from)
            .collect();

        EnvironmentInfo {
            os,
            arch,
            rustc_version,
            available_binaries,
        }
    }
}

impl DefaultEnvironmentProbe {
    fn get_rustc_version() -> Option<String> {
        Command::new("rustc")
            .arg("--version")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_environment_probe_exists() {
        let probe = DefaultEnvironmentProbe;
        let _info = probe.get_info();
    }

    #[test]
    fn test_environment_probe_trait_is_implemented() {
        let probe: DefaultEnvironmentProbe = Default::default();
        let _ = probe.check_binary("ls");
        let info = probe.get_info();
        assert!(!info.os.is_empty());
    }

    #[test]
    fn test_check_binary_returns_true_for_existing_binaries() {
        let probe = DefaultEnvironmentProbe;
        #[cfg(target_os = "windows")]
        let result = probe.check_binary("where");
        #[cfg(not(target_os = "windows"))]
        let result = probe.check_binary("which");
        assert!(result);
    }

    #[test]
    fn test_get_info_returns_environment_info() {
        let probe = DefaultEnvironmentProbe;
        let info = probe.get_info();
        assert_eq!(info.os, env::consts::OS);
        assert_eq!(info.arch, env::consts::ARCH);
    }

    #[test]
    fn test_get_info_contains_rustc_version() {
        let probe = DefaultEnvironmentProbe;
        let info = probe.get_info();
        if let Some(ref version) = info.rustc_version {
            assert!(version.starts_with("rustc ") || version.starts_with("rustc"));
        }
    }

    #[test]
    fn test_default_implementation_exists() {
        let _probe: DefaultEnvironmentProbe = Default::default();
    }

    #[test]
    fn test_check_binary_with_nonexistent_binary_returns_false() {
        let probe = DefaultEnvironmentProbe;
        assert!(!probe.check_binary("this_binary_definitely_does_not_exist_12345"));
    }

    #[test]
    fn test_trait_objects_can_be_created_and_used_polymorphically() {
        let probe: Box<dyn EnvironmentProbe> = Box::new(DefaultEnvironmentProbe::default());
        assert!(!probe.check_binary("this_binary_definitely_does_not_exist_12345"));
        let info = probe.get_info();
        assert_eq!(info.os, env::consts::OS);
    }
}
