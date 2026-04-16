pub fn normalize(input: &str) -> String {
    input.replace("\r\n", "\n").replace("\r", "\n")
}

pub fn to_crlf(input: &str) -> String {
    input.replace("\n", "\r\n")
}

pub fn to_lf(input: &str) -> String {
    normalize(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_crlf_to_lf() {
        assert_eq!(normalize("hello\r\nworld"), "hello\nworld");
    }

    #[test]
    fn test_normalize_cr_to_lf() {
        assert_eq!(normalize("hello\rworld"), "hello\nworld");
    }

    #[test]
    fn test_normalize_already_lf() {
        assert_eq!(normalize("hello\nworld"), "hello\nworld");
    }

    #[test]
    fn test_to_crlf() {
        assert_eq!(to_crlf("hello\nworld"), "hello\r\nworld");
    }

    #[test]
    fn test_to_lf() {
        assert_eq!(to_lf("hello\r\nworld"), "hello\nworld");
    }

    #[test]
    fn test_normalize_empty() {
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn test_normalize_mixed_endings() {
        assert_eq!(
            normalize("line1\r\nline2\rline3\n"),
            "line1\nline2\nline3\n"
        );
    }
}
