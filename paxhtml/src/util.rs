//! Utility functions.

/// Slugify a string.
///
/// This converts a string to a slug by lowercasing it, replacing spaces and
/// dashes with a single dash, and removing any characters that are not alphanumeric
/// or a dash.
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' => '-',
            _ => ' ',
        })
        .filter(|c| *c != ' ')
        .collect()
}
