pub fn normalize(input: &str) -> String {
    collapse_lines(&trim_lines(input))
}

pub fn trim_lines(input: &str) -> String {
    input
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn collapse_lines(input: &str) -> String {
    input
        .lines()
        .map(collapse_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn collapse_whitespace(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut last_was_space = false;

    for c in input.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_trims() {
        assert_eq!(normalize("  hello  "), "hello");
    }

    #[test]
    fn test_normalize_collapse_tabs() {
        assert_eq!(normalize("hello\t\tworld"), "hello world");
    }

    #[test]
    fn test_normalize_collapse_multiple_spaces() {
        assert_eq!(normalize("hello    world"), "hello world");
    }

    #[test]
    fn test_normalize_collapse_lines() {
        assert_eq!(normalize("line1\nline2"), "line1 line2");
    }

    #[test]
    fn test_normalize_empty() {
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn test_normalize_multiline_with_whitespace() {
        assert_eq!(normalize("  line1  \n  line2  \n"), "line1 line2");
    }

    #[test]
    fn test_trim_lines() {
        assert_eq!(trim_lines("  hello  \n  world  "), "hello\nworld");
    }

    #[test]
    fn test_collapse_lines() {
        assert_eq!(collapse_lines("line1\nline2"), "line1 line2");
    }

    #[test]
    fn test_collapse_whitespace() {
        assert_eq!(collapse_whitespace("hello    world"), "hello world");
        assert_eq!(collapse_whitespace("foo\t\tbar"), "foo bar");
    }

    #[test]
    fn test_collapse_whitespace_no_change() {
        assert_eq!(collapse_whitespace("hello world"), "hello world");
    }
}
