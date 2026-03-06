use anyhow::{Context, Result};
use std::process::Command;

/// Detect owner and repo from the git remote origin URL.
/// Returns (owner, repo) or an error if it can't be parsed.
pub fn detect_owner_repo() -> Result<(String, String)> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("failed to run 'git remote get-url origin'")?;

    if !output.status.success() {
        anyhow::bail!("git remote get-url origin failed: not a git repository or no origin remote");
    }

    let url = String::from_utf8(output.stdout)
        .context("git remote URL is not valid UTF-8")?
        .trim()
        .to_string();

    parse_remote_url(&url)
}

/// Parse a git remote URL into (owner, repo).
/// Supports SSH (`git@github.com:owner/repo.git`) and
/// HTTPS (`https://github.com/owner/repo.git`) formats.
pub fn parse_remote_url(url: &str) -> Result<(String, String)> {
    let path = if let Some(rest) = url.strip_prefix("git@") {
        // SSH format: git@github.com:owner/repo.git
        rest.split_once(':')
            .map(|(_, path)| path)
            .context("invalid SSH remote URL format")?
    } else if url.starts_with("https://") || url.starts_with("http://") {
        // HTTPS format: https://github.com/owner/repo.git
        let without_scheme = url
            .split("://")
            .nth(1)
            .context("invalid URL: missing scheme separator")?;
        // Skip the host part
        without_scheme
            .split_once('/')
            .map(|(_, path)| path)
            .context("invalid HTTPS remote URL format")?
    } else {
        anyhow::bail!("unsupported remote URL format: {url}");
    };

    // Remove trailing .git if present
    let path = path.strip_suffix(".git").unwrap_or(path);

    let parts: Vec<&str> = path.splitn(3, '/').collect();
    if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
        anyhow::bail!("could not extract owner/repo from remote URL: {url}");
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ssh_url() {
        let (owner, repo) = parse_remote_url("git@github.com:mpecan/cov-info.git").unwrap();
        assert_eq!(owner, "mpecan");
        assert_eq!(repo, "cov-info");
    }

    #[test]
    fn parse_ssh_url_no_dot_git() {
        let (owner, repo) = parse_remote_url("git@github.com:mpecan/cov-info").unwrap();
        assert_eq!(owner, "mpecan");
        assert_eq!(repo, "cov-info");
    }

    #[test]
    fn parse_https_url() {
        let (owner, repo) = parse_remote_url("https://github.com/mpecan/cov-info.git").unwrap();
        assert_eq!(owner, "mpecan");
        assert_eq!(repo, "cov-info");
    }

    #[test]
    fn parse_https_url_no_dot_git() {
        let (owner, repo) = parse_remote_url("https://github.com/mpecan/cov-info").unwrap();
        assert_eq!(owner, "mpecan");
        assert_eq!(repo, "cov-info");
    }

    #[test]
    fn parse_gitlab_ssh() {
        let (owner, repo) = parse_remote_url("git@gitlab.com:myorg/myproject.git").unwrap();
        assert_eq!(owner, "myorg");
        assert_eq!(repo, "myproject");
    }

    #[test]
    fn parse_gitlab_https() {
        let (owner, repo) = parse_remote_url("https://gitlab.com/myorg/myproject.git").unwrap();
        assert_eq!(owner, "myorg");
        assert_eq!(repo, "myproject");
    }

    #[test]
    fn parse_http_url() {
        let (owner, repo) = parse_remote_url("http://github.com/owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn unsupported_format_errors() {
        let result = parse_remote_url("svn://example.com/repo");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_ssh_format_errors() {
        let result = parse_remote_url("git@github.com/missing-colon.git");
        assert!(result.is_err());
    }
}
