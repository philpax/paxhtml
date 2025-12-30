//! Utility functions.

use bumpalo::collections::String as BumpString;
use bumpalo::Bump;

/// Slugify a string.
///
/// This converts a string to a slug by lowercasing it, replacing spaces and
/// dashes with a single dash, and removing any characters that are not alphanumeric
/// or a dash.
pub fn slugify<'bump>(bump: &'bump Bump, s: &str) -> BumpString<'bump> {
    let mut result = BumpString::new_in(bump);
    for c in s.to_lowercase().chars() {
        match c {
            'a'..='z' | '0'..='9' => result.push(c),
            ' ' | '-' => result.push('-'),
            _ => {}
        }
    }
    result
}
