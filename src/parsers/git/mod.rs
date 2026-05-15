//! Git ecosystem parsers
//!
//! This module provides parsers for Git-based dependencies:
//! - Git submodules: .gitmodules file parsing and detection
//! - Commit SHA resolution: Get exact commit SHAs for submodules
//! - URL parsing: Extract owner/repo from various Git URL formats
//!
//! ## Features (v1.0.0)
//! - Parse .gitmodules INI-like format
//! - Resolve commit SHAs via `git ls-tree`
//! - Support for GitHub, GitLab, Bitbucket, and self-hosted Git servers
//! - Support for nested namespaces (e.g., GitLab groups/subgroups)
//!
//! ## Future Enhancements
//! - Recursive scanning of dependencies within submodules (v1.0.1+)
//! - Circular reference detection (v1.0.1+)

pub mod commit_resolver;
pub mod submodules;
pub mod url_parser;

// Re-export main types and functions
pub use commit_resolver::{is_git_available, resolve_submodule_commits};
pub use submodules::parse_gitmodules;
pub use url_parser::{is_git_repo_url, parse_git_url, GitHostType};
