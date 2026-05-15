//! Git URL parser
//!
//! Parses Git repository URLs to extract owner and repository name
//! for purl generation.
//!
//! ## Supported URL formats
//! - HTTPS: `https://github.com/owner/repo.git`
//! - SSH: `git@github.com:owner/repo.git`
//! - HTTP: `http://github.com/owner/repo`
//!
//! ## Supported hosts
//! - GitHub (`pkg:github/`)
//! - GitLab (`pkg:gitlab/`)
//! - Bitbucket (`pkg:bitbucket/`)
//! - Self-hosted / Generic (`pkg:generic/`)

/// Git host type for purl generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitHostType {
    /// GitHub (github.com)
    GitHub,
    /// GitLab (gitlab.com or self-hosted GitLab)
    GitLab,
    /// Bitbucket (bitbucket.org)
    Bitbucket,
    /// Self-hosted or unknown Git server
    Generic,
}

impl GitHostType {
    /// Get the purl type string for this host
    pub fn purl_type(&self) -> &'static str {
        match self {
            GitHostType::GitHub => "github",
            GitHostType::GitLab => "gitlab",
            GitHostType::Bitbucket => "bitbucket",
            GitHostType::Generic => "generic",
        }
    }
}

/// Parsed Git repository information
#[derive(Debug, Clone)]
pub struct GitRepoInfo {
    /// Type of Git host
    pub host_type: GitHostType,
    /// Repository owner (user or organization)
    pub owner: String,
    /// Repository name
    pub repo: String,
}

impl GitRepoInfo {
    /// Generate a purl (package URL) for this repository
    ///
    /// # Arguments
    /// * `version` - Version string (usually a commit SHA or tag)
    ///
    /// # Returns
    /// A purl string like `pkg:github/owner/repo@version`
    pub fn to_purl(&self, version: &str) -> String {
        format!(
            "pkg:{}/{}/{}@{}",
            self.host_type.purl_type(),
            self.owner,
            self.repo,
            version
        )
    }
}

/// Check if a URL looks like a Git repository URL
///
/// Returns `true` if the URL appears to be a Git repository (not an archive download).
///
/// # Arguments
/// * `url` - The URL to check
///
/// # Returns
/// `true` if the URL looks like a Git repo, `false` if it's likely an archive download
///
/// # Examples
/// ```
/// use radeis_sc2sbom::parsers::git::url_parser::is_git_repo_url;
///
/// // Git repository URLs
/// assert!(is_git_repo_url("https://github.com/owner/repo.git"));
/// assert!(is_git_repo_url("git@github.com:owner/repo.git"));
/// assert!(is_git_repo_url("https://github.com/owner/repo")); // implicit .git
///
/// // Archive download URLs (NOT Git repos)
/// assert!(!is_git_repo_url("https://github.com/owner/repo/releases/download/v1.0.0/file.tar.gz"));
/// assert!(!is_git_repo_url("https://github.com/owner/repo/archive/refs/tags/v1.0.0.tar.gz"));
/// ```
pub fn is_git_repo_url(url: &str) -> bool {
    let url = url.trim();

    // SSH URLs are always Git repos
    if url.starts_with("git@") {
        return true;
    }

    // URLs ending with .git are Git repos
    if url.ends_with(".git") {
        return true;
    }

    // Archive/release download URLs are NOT Git repos
    if url.contains("/releases/download/")
        || url.contains("/archive/")
        || url.contains("/download/")
        || url.ends_with(".tar.gz")
        || url.ends_with(".tgz")
        || url.ends_with(".zip")
        || url.ends_with(".tar.bz2")
        || url.ends_with(".tar.xz")
    {
        return false;
    }

    // HTTPS/HTTP URL without .git suffix that looks like owner/repo pattern.
    // Example: https://github.com/owner/repo (common Git clone format)
    // Also support nested groups (e.g., GitLab: host/group/subgroup/project)
    // while rejecting common non-repo subpaths like /tree/, /blob/, etc.
    if url.starts_with("http://") || url.starts_with("https://") {
        let without_protocol = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            // Normalize away any trailing slashes so that
            // https://github.com/owner/repo/ is treated like /owner/repo
            .trim_end_matches('/');

        // Reject known non-repo URL patterns that point to files, commits,
        // issues, or other sub-resources rather than the repo root.
        if without_protocol.contains("/tree/")
            || without_protocol.contains("/blob/")
            || without_protocol.contains("/raw/")
            || without_protocol.contains("/commit/")
            || without_protocol.contains("/commits/")
            || without_protocol.contains("/issues/")
            || without_protocol.contains("/pull/")
            || without_protocol.contains("/pulls/")
            || without_protocol.contains("/merge_requests/")
            || without_protocol.contains("/compare/")
            || without_protocol.contains("/releases/")
            || without_protocol.contains("/tags/")
        {
            return false;
        }

        let parts: Vec<&str> = without_protocol.split('/').collect();

        // Require at least host/owner/repo (3 parts). Additional path
        // components are allowed to support nested groups (e.g. GitLab),
        // but URLs with too few components are not repositories.
        if parts.len() < 3 {
            return false;
        }

        // Reject URLs that point to files (have file extensions)
        if let Some(last_part) = parts.last() {
            // Check if the last component has a file extension
            // (but allow common Git host patterns like .git or .io)
            if last_part.contains('.')
                && !last_part.ends_with(".git")
                && !last_part.ends_with(".io")
            {
                // Has an extension and it's not .git or .io, likely a file
                return false;
            }
        }

        // Very deep paths (> 5 components) are unlikely to be Git repos
        // This catches things like /path/to/some/file.txt while still allowing
        // GitLab nested groups like gitlab.com/group/subgroup/project
        if parts.len() > 5 {
            return false;
        }

        return true;
    }

    false
}

/// Parse a Git URL into its components
///
/// Handles both HTTPS and SSH formats for various Git hosts.
///
/// **Note**: This function attempts to parse ANY URL that looks like it might
/// contain repository information. Use `is_git_repo_url()` first to filter out
/// archive download URLs before calling this function.
///
/// # Arguments
/// * `url` - The Git repository URL
///
/// # Returns
/// `Some(GitRepoInfo)` if the URL could be parsed, `None` otherwise
///
/// # Examples
/// ```
/// use radeis_sc2sbom::parsers::git::url_parser::{parse_git_url, is_git_repo_url};
///
/// // HTTPS URL
/// let url = "https://github.com/owner/repo.git";
/// if is_git_repo_url(url) {
///     let info = parse_git_url(url).unwrap();
///     assert_eq!(info.owner, "owner");
///     assert_eq!(info.repo, "repo");
/// }
///
/// // SSH URL
/// let info = parse_git_url("git@github.com:owner/repo.git").unwrap();
/// assert_eq!(info.owner, "owner");
/// assert_eq!(info.repo, "repo");
/// ```
pub fn parse_git_url(url: &str) -> Option<GitRepoInfo> {
    let url = url.trim().trim_end_matches(".git");

    // SSH format: git@github.com:owner/repo
    if url.starts_with("git@") {
        return parse_ssh_url(url);
    }

    // HTTPS/HTTP format: https://github.com/owner/repo
    if url.starts_with("http://") || url.starts_with("https://") {
        return parse_https_url(url);
    }

    // Unknown format
    None
}

/// Parse an SSH-style Git URL
fn parse_ssh_url(url: &str) -> Option<GitRepoInfo> {
    // Format: git@github.com:owner/repo or git@gitlab.com:group/subgroup/repo
    let parts: Vec<&str> = url.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let host = parts[0].trim_start_matches("git@");
    let path = parts[1];

    // Split path into namespace/owner and repo
    let path_parts: Vec<&str> = path.split('/').collect();
    if path_parts.len() < 2 {
        return None;
    }

    // Use all but the last segment as the namespace/owner (preserves nested groups)
    // This handles GitLab's nested groups: group/subgroup/project
    let owner = path_parts[..path_parts.len() - 1].join("/");
    let repo = path_parts[path_parts.len() - 1].to_string();

    Some(GitRepoInfo {
        host_type: detect_host_type(host),
        owner,
        repo,
    })
}

/// Parse an HTTPS-style Git URL
fn parse_https_url(url: &str) -> Option<GitRepoInfo> {
    let without_protocol = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    let parts: Vec<&str> = without_protocol.split('/').collect();
    if parts.len() < 3 {
        return None;
    }

    let host = parts[0];

    // Use all but the last segment as the namespace/owner (preserves nested groups)
    // This handles GitLab's nested groups: https://gitlab.com/group/subgroup/project
    let owner = parts[1..parts.len() - 1].join("/");
    let repo = parts[parts.len() - 1].to_string();

    Some(GitRepoInfo {
        host_type: detect_host_type(host),
        owner,
        repo,
    })
}

/// Detect the Git host type from the hostname
fn detect_host_type(host: &str) -> GitHostType {
    let host_lower = host.to_lowercase();

    if host_lower.contains("github.com") || host_lower == "github" {
        GitHostType::GitHub
    } else if host_lower.contains("gitlab.com") || host_lower.contains("gitlab") {
        GitHostType::GitLab
    } else if host_lower.contains("bitbucket.org") || host_lower.contains("bitbucket") {
        GitHostType::Bitbucket
    } else {
        GitHostType::Generic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_https() {
        let info = parse_git_url("https://github.com/nlohmann/json.git").unwrap();
        assert_eq!(info.host_type, GitHostType::GitHub);
        assert_eq!(info.owner, "nlohmann");
        assert_eq!(info.repo, "json");
    }

    #[test]
    fn test_parse_github_https_no_git_suffix() {
        let info = parse_git_url("https://github.com/nlohmann/json").unwrap();
        assert_eq!(info.host_type, GitHostType::GitHub);
        assert_eq!(info.owner, "nlohmann");
        assert_eq!(info.repo, "json");
    }

    #[test]
    fn test_parse_github_ssh() {
        let info = parse_git_url("git@github.com:nlohmann/json.git").unwrap();
        assert_eq!(info.host_type, GitHostType::GitHub);
        assert_eq!(info.owner, "nlohmann");
        assert_eq!(info.repo, "json");
    }

    #[test]
    fn test_parse_gitlab_https() {
        let info = parse_git_url("https://gitlab.com/gabime/spdlog.git").unwrap();
        assert_eq!(info.host_type, GitHostType::GitLab);
        assert_eq!(info.owner, "gabime");
        assert_eq!(info.repo, "spdlog");
    }

    #[test]
    fn test_parse_gitlab_ssh() {
        let info = parse_git_url("git@gitlab.com:gabime/spdlog.git").unwrap();
        assert_eq!(info.host_type, GitHostType::GitLab);
        assert_eq!(info.owner, "gabime");
        assert_eq!(info.repo, "spdlog");
    }

    #[test]
    fn test_parse_bitbucket_https() {
        let info = parse_git_url("https://bitbucket.org/owner/repo.git").unwrap();
        assert_eq!(info.host_type, GitHostType::Bitbucket);
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_parse_self_hosted() {
        let info = parse_git_url("https://git.company.com/team/project.git").unwrap();
        assert_eq!(info.host_type, GitHostType::Generic);
        assert_eq!(info.owner, "team");
        assert_eq!(info.repo, "project");
    }

    #[test]
    fn test_parse_self_hosted_ssh() {
        let info = parse_git_url("git@git.company.com:team/project.git").unwrap();
        assert_eq!(info.host_type, GitHostType::Generic);
        assert_eq!(info.owner, "team");
        assert_eq!(info.repo, "project");
    }

    #[test]
    fn test_parse_nested_gitlab_groups() {
        // GitLab allows nested groups: group/subgroup/repo
        // v1.0.0: Now preserves full namespace path
        let info = parse_git_url("https://gitlab.com/group/subgroup/project.git").unwrap();
        assert_eq!(info.host_type, GitHostType::GitLab);
        assert_eq!(info.owner, "group/subgroup");
        assert_eq!(info.repo, "project");
    }

    #[test]
    fn test_to_purl() {
        let info = GitRepoInfo {
            host_type: GitHostType::GitHub,
            owner: "nlohmann".to_string(),
            repo: "json".to_string(),
        };

        assert_eq!(info.to_purl("v3.11.2"), "pkg:github/nlohmann/json@v3.11.2");
    }

    #[test]
    fn test_to_purl_commit_sha() {
        let info = GitRepoInfo {
            host_type: GitHostType::GitHub,
            owner: "nlohmann".to_string(),
            repo: "json".to_string(),
        };

        assert_eq!(info.to_purl("abc1234"), "pkg:github/nlohmann/json@abc1234");
    }

    #[test]
    fn test_invalid_url() {
        assert!(parse_git_url("not a url").is_none());
        assert!(parse_git_url("").is_none());
        assert!(parse_git_url("ftp://example.com/repo").is_none());
    }

    #[test]
    fn test_purl_type() {
        assert_eq!(GitHostType::GitHub.purl_type(), "github");
        assert_eq!(GitHostType::GitLab.purl_type(), "gitlab");
        assert_eq!(GitHostType::Bitbucket.purl_type(), "bitbucket");
        assert_eq!(GitHostType::Generic.purl_type(), "generic");
    }

    #[test]
    fn test_is_git_repo_url_git_repos() {
        // URLs that ARE Git repositories
        assert!(is_git_repo_url("https://github.com/owner/repo.git"));
        assert!(is_git_repo_url("git@github.com:owner/repo.git"));
        assert!(is_git_repo_url("https://github.com/owner/repo")); // implicit .git
                                                                   // Trailing-slash repository URL (should still be treated as a Git repo)
        assert!(is_git_repo_url("https://github.com/owner/repo/"));

        assert!(is_git_repo_url("https://gitlab.com/group/project.git"));
        assert!(is_git_repo_url("git@gitlab.com:group/project.git"));
        // GitLab nested-group URLs without .git suffix (should be treated as Git repos)
        assert!(is_git_repo_url("https://gitlab.com/group/subgroup/project"));
        assert!(is_git_repo_url(
            "https://gitlab.com/group/subgroup/project/"
        ));
    }

    #[test]
    fn test_is_git_repo_url_archives() {
        // URLs that are NOT Git repositories (archive downloads)
        assert!(!is_git_repo_url(
            "https://github.com/fmtlib/fmt/releases/download/9.1.0/fmt-9.1.0.zip"
        ));
        assert!(!is_git_repo_url(
            "https://github.com/owner/repo/archive/refs/tags/v1.0.0.tar.gz"
        ));
        assert!(!is_git_repo_url(
            "https://example.com/download/project-1.2.3.tar.gz"
        ));
        assert!(!is_git_repo_url(
            "https://github.com/owner/repo/archive/master.zip"
        ));
        assert!(!is_git_repo_url("https://example.com/files/library.tgz"));
    }

    #[test]
    fn test_is_git_repo_url_deep_paths() {
        // URLs with deep paths are NOT Git repos (file paths or downloads)
        assert!(!is_git_repo_url(
            "https://github.com/owner/repo/tree/main/docs"
        ));
        assert!(!is_git_repo_url(
            "https://example.com/path/to/some/file.txt"
        ));
    }
}
