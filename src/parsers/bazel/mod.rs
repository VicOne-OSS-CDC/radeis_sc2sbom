//! Bazel build system parser
//!
//! Parses WORKSPACE/WORKSPACE.bazel and MODULE.bazel files to extract dependencies.
//!
//! ## Supported formats:
//! - `WORKSPACE`: http_archive(), git_repository(), local_repository()
//! - `MODULE.bazel`: bazel_dep() (Bazel 6.0+ bzlmod)
//!
//! ## Ecosystems:
//! - `"bazel"`: For WORKSPACE dependencies
//! - `"bazel-bzlmod"`: For MODULE.bazel dependencies (Bazel 6.0+)
//!
//! ## Limitations (v1.0.4):
//! - BUILD file parsing not included (WORKSPACE only)
//! - No macro expansion
//! - No transitive dependency computation
//! - Rule repositories may appear in output

pub mod module;
pub mod workspace;

pub use module::parse_module_bazel;
pub use workspace::parse_workspace;

/// Find the position of the matching closing parenthesis
/// Assumes the string starts after the opening '(' has been consumed
///
/// # Arguments
/// * `s` - String slice starting after the opening parenthesis
///
/// # Returns
/// * `Some(usize)` - Position of the matching closing parenthesis
/// * `None` - No matching closing parenthesis found (unbalanced)
///
/// # Known Limitations
/// This function does not handle parentheses inside string literals. For example:
/// `name = "my(project)"` would incorrectly count the '(' inside the string.
/// In practice, this is rarely an issue in Bazel WORKSPACE/MODULE files as
/// parentheses in dependency names or URLs are uncommon.
///
/// # Examples
/// ```
/// # use radeis_sc2sbom::parsers::bazel::find_matching_paren;
/// assert_eq!(find_matching_paren("name = \"foo\")"), Some(12));
/// assert_eq!(find_matching_paren("nested(x), y)"), Some(12));
/// assert_eq!(find_matching_paren("unbalanced("), None);
/// ```
pub fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 1; // Start at 1 since opening paren is already consumed
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}
