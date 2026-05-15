//! Git commit SHA resolver
//!
//! Resolves commit SHAs for Git submodules using `git ls-tree`.
//!
//! ## Features
//! - Check if git command is available
//! - Resolve commit SHAs for submodules
//! - Handle uninitialized submodules gracefully

use anyhow::Result;
use std::path::Path;
use std::process::Command;

use super::submodules::GitSubmodule;

/// Check if the git command is available on the system
///
/// # Returns
/// `true` if git is available, `false` otherwise
pub fn is_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Resolve commit SHAs for a list of submodules
///
/// Uses `git ls-tree HEAD <path>` to get the exact commit SHA
/// that each submodule is pinned to.
///
/// # Arguments
/// * `repo_root` - Path to the repository root
/// * `submodules` - Mutable slice of submodules to update with commit SHAs
///
/// # Example
/// ```ignore
/// let mut submodules = parse_gitmodules(Path::new(".gitmodules"))?;
/// resolve_submodule_commits(Path::new("."), &mut submodules)?;
/// for sm in &submodules {
///     println!("{}: {}", sm.name, sm.commit_sha.as_deref().unwrap_or("uninitialized"));
/// }
/// ```
pub fn resolve_submodule_commits(repo_root: &Path, submodules: &mut [GitSubmodule]) -> Result<()> {
    for submodule in submodules.iter_mut() {
        // Run: git ls-tree HEAD <submodule_path>
        // Output: 160000 commit <sha>\t<path>
        let output = Command::new("git")
            .current_dir(repo_root)
            .args(["ls-tree", "HEAD", &submodule.path.to_string_lossy()])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(sha) = parse_ls_tree_output(&stdout) {
                    submodule.commit_sha = Some(sha);
                } else {
                    // Path exists but no commit SHA (edge case)
                    submodule.commit_sha = None;
                }
            }
            Ok(_) => {
                // Command failed (submodule not initialized or other error)
                submodule.commit_sha = None;
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to resolve commit for submodule {}: {}",
                    submodule.name, e
                );
                submodule.commit_sha = None;
            }
        }
    }

    Ok(())
}

/// Parse the output of `git ls-tree` to extract the commit SHA
///
/// # Format
/// `160000 commit <sha>\t<path>`
///
/// # Arguments
/// * `output` - The stdout from git ls-tree command
///
/// # Returns
/// The commit SHA if found, None otherwise
fn parse_ls_tree_output(output: &str) -> Option<String> {
    // Format: "160000 commit <sha>\t<path>"
    // The mode 160000 indicates a gitlink (submodule reference)
    let line = output.lines().next()?;
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() >= 3 && parts[1] == "commit" {
        Some(parts[2].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ls_tree_output() {
        let output = "160000 commit abc123def456789012345678901234567890abcd\tlibs/json\n";
        let sha = parse_ls_tree_output(output);
        assert_eq!(
            sha,
            Some("abc123def456789012345678901234567890abcd".to_string())
        );
    }

    #[test]
    fn test_parse_ls_tree_output_empty() {
        let output = "";
        let sha = parse_ls_tree_output(output);
        assert_eq!(sha, None);
    }

    #[test]
    fn test_parse_ls_tree_output_invalid() {
        let output = "100644 blob abc123\tsome_file.txt\n";
        let sha = parse_ls_tree_output(output);
        assert_eq!(sha, None);
    }

    #[test]
    fn test_is_git_available() {
        // This test will pass on systems with git installed
        // and fail (or return false) on systems without git
        let available = is_git_available();
        // Just verify it doesn't panic
        println!("Git available: {}", available);
    }
}
