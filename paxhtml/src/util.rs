//! Utility functions.

/// Slugify a string.
///
/// This converts a string to a slug by lowercasing it, replacing spaces and
/// dashes with a single dash, and removing any characters that are not alphanumeric
/// or a dash.
pub fn slugify(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_dash = false;
    for c in s.to_lowercase().chars() {
        match c {
            'a'..='z' | '0'..='9' => {
                prev_dash = false;
                result.push(c);
            }
            ' ' | '-' if !prev_dash && !result.is_empty() => {
                prev_dash = true;
                result.push('-');
            }
            _ => {}
        }
    }
    if result.ends_with('-') {
        result.pop();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugify_dash_with_spaces() {
        assert_eq!(slugify("A - B"), "a-b");
    }

    #[test]
    fn test_slugify_multiple_dashes_and_spaces() {
        assert_eq!(slugify("A  --  B"), "a-b");
    }

    #[test]
    fn test_slugify_existing_dashes() {
        assert_eq!(slugify("my-title"), "my-title");
    }

    #[test]
    fn test_slugify_special_characters() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
    }

    #[test]
    fn test_slugify_leading_trailing_spaces() {
        assert_eq!(slugify(" Hello World "), "hello-world");
    }

    #[test]
    fn test_slugify_numbers() {
        assert_eq!(slugify("Title 123"), "title-123");
    }

    #[test]
    fn test_slugify_only_special_characters() {
        assert_eq!(slugify("!!!"), "");
    }

    #[test]
    fn test_slugify_unicode() {
        assert_eq!(slugify("Héllo Wörld"), "hllo-wrld");
    }

    #[test]
    fn test_slugify_dash_separated_title() {
        assert_eq!(
            slugify("BLUE - A generic build-system crafted entirely in Guile"),
            "blue-a-generic-build-system-crafted-entirely-in-guile"
        );
    }
}
