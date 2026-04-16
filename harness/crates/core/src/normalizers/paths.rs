pub fn normalize(input: &str) -> String {
    #[cfg(windows)]
    {
        input.replace('\\', "/")
    }
    #[cfg(not(windows))]
    {
        input.to_string()
    }
}

#[cfg(windows)]
pub fn normalize_to_platform(input: &str) -> String {
    input.replace('/', "\\")
}

#[cfg(not(windows))]
pub fn normalize_to_platform(input: &str) -> String {
    input.to_string()
}

pub fn expand_home(input: &str) -> String {
    if input.starts_with("~/") || input == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return input.replace("~", &home);
        }
    }
    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_no_change_on_unix() {
        #[cfg(not(windows))]
        {
            assert_eq!(normalize("/home/user/file.txt"), "/home/user/file.txt");
        }
    }

    #[test]
    fn test_normalize_converts_backslash_on_windows() {
        #[cfg(windows)]
        {
            assert_eq!(
                normalize("C:\\Users\\test\\file.txt"),
                "C:/Users/test/file.txt"
            );
        }
    }

    #[test]
    fn test_normalize_to_platform_no_change_on_unix() {
        #[cfg(not(windows))]
        {
            assert_eq!(
                normalize_to_platform("/home/user/file.txt"),
                "/home/user/file.txt"
            );
        }
    }

    #[test]
    fn test_expand_home_tilde() {
        let result = expand_home("~/test");
        assert!(!result.starts_with("~/"));
        assert!(result.contains("/test"));
    }

    #[test]
    fn test_expand_home_exact_tilde() {
        let result = expand_home("~");
        assert!(!result.starts_with("~"));
    }

    #[test]
    fn test_expand_home_no_tilde() {
        let result = expand_home("/absolute/path");
        assert_eq!(result, "/absolute/path");
    }
}
