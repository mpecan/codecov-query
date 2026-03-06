use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Service {
    Github,
    Gitlab,
    Bitbucket,
    GithubEnterprise,
    GitlabEnterprise,
    BitbucketServer,
}

impl Service {
    pub const fn as_api_str(&self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Bitbucket => "bitbucket",
            Self::GithubEnterprise => "github_enterprise",
            Self::GitlabEnterprise => "gitlab_enterprise",
            Self::BitbucketServer => "bitbucket_server",
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

#[derive(Parser, Debug)]
#[command(name = "codecov-query", about = "Query the Codecov API v2")]
pub struct Cli {
    /// Codecov API token (required, can also be set via `CODECOV_TOKEN` env var)
    #[arg(long, env = "CODECOV_TOKEN", global = true)]
    pub token: Option<String>,

    /// Git hosting service
    #[arg(long, default_value = "github", global = true)]
    pub service: Service,

    /// Repository owner (auto-detected from git remote if not provided)
    #[arg(short, long, global = true)]
    pub owner: Option<String>,

    /// Repository name (auto-detected from git remote if not provided)
    #[arg(short, long, global = true)]
    pub repo: Option<String>,

    /// Output format
    #[arg(long, default_value = "json", global = true)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List repositories for an owner
    Repos {
        /// Filter by active status
        #[arg(long)]
        active: Option<bool>,
        /// Search repositories by name
        #[arg(long)]
        search: Option<String>,
        /// Filter by repository names (comma-separated)
        #[arg(long)]
        names: Option<String>,
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Page size
        #[arg(long, alias = "page-size")]
        page_size: Option<u32>,
    },

    /// Get details for a specific repository
    Repo,

    /// Get coverage totals for a repository
    Totals {
        /// Commit SHA
        #[arg(long)]
        sha: Option<String>,
        /// Branch name
        #[arg(long)]
        branch: Option<String>,
        /// File path filter
        #[arg(long)]
        path: Option<String>,
        /// Flag filter
        #[arg(long)]
        flag: Option<String>,
        /// Component ID filter
        #[arg(long, alias = "component-id")]
        component_id: Option<String>,
    },

    /// List commits for a repository
    Commits {
        /// Branch name
        #[arg(long)]
        branch: Option<String>,
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Page size
        #[arg(long, alias = "page-size")]
        page_size: Option<u32>,
    },

    /// Get details for a specific commit
    Commit {
        /// Commit SHA
        commitid: String,
    },

    /// List branches for a repository
    Branches {
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Page size
        #[arg(long, alias = "page-size")]
        page_size: Option<u32>,
    },

    /// Get details for a specific branch
    Branch {
        /// Branch name
        name: String,
    },

    /// List pull requests for a repository
    Pulls {
        /// Pull request state filter
        #[arg(long)]
        state: Option<String>,
        /// Start date filter
        #[arg(long, alias = "start-date")]
        start_date: Option<String>,
        /// Ordering
        #[arg(long)]
        ordering: Option<String>,
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Page size
        #[arg(long, alias = "page-size")]
        page_size: Option<u32>,
    },

    /// Get details for a specific pull request
    Pull {
        /// Pull request ID
        pullid: u64,
    },

    /// Compare coverage between two refs or for a pull request
    Compare {
        /// Base commit SHA (use with --head)
        #[arg(long, requires = "head", conflicts_with = "pullid")]
        base: Option<String>,
        /// Head commit SHA (use with --base)
        #[arg(long, requires = "base", conflicts_with = "pullid")]
        head: Option<String>,
        /// Pull request ID (alternative to --base/--head)
        #[arg(long, conflicts_with_all = ["base", "head"])]
        pullid: Option<u64>,
    },

    /// Get file-level coverage report
    FileReport {
        /// File path
        path: String,
        /// Commit SHA
        #[arg(long)]
        sha: Option<String>,
        /// Branch name
        #[arg(long)]
        branch: Option<String>,
    },

    /// List flags for a repository
    Flags {
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Page size
        #[arg(long, alias = "page-size")]
        page_size: Option<u32>,
    },

    /// List components for a repository
    Components,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_repos_command() {
        let cli = Cli::parse_from([
            "cov-info",
            "--token",
            "test-token",
            "--owner",
            "myowner",
            "repos",
            "--active",
            "true",
            "--page",
            "2",
        ]);
        assert!(matches!(cli.command, Command::Repos { .. }));
        if let Command::Repos { active, page, .. } = cli.command {
            assert_eq!(active, Some(true));
            assert_eq!(page, Some(2));
        }
    }

    #[test]
    fn parse_totals_command() {
        let cli = Cli::parse_from([
            "cov-info", "--token", "t", "totals", "--branch", "main", "--flag", "unit",
        ]);
        if let Command::Totals { branch, flag, .. } = cli.command {
            assert_eq!(branch.as_deref(), Some("main"));
            assert_eq!(flag.as_deref(), Some("unit"));
        } else {
            panic!("expected Totals");
        }
    }

    #[test]
    fn parse_commit_command() {
        let cli = Cli::parse_from(["cov-info", "--token", "t", "commit", "abc123"]);
        if let Command::Commit { commitid } = cli.command {
            assert_eq!(commitid, "abc123");
        } else {
            panic!("expected Commit");
        }
    }

    #[test]
    fn parse_compare_with_pullid() {
        let cli = Cli::parse_from(["cov-info", "--token", "t", "compare", "--pullid", "42"]);
        if let Command::Compare { pullid, base, head } = cli.command {
            assert_eq!(pullid, Some(42));
            assert!(base.is_none());
            assert!(head.is_none());
        } else {
            panic!("expected Compare");
        }
    }

    #[test]
    fn parse_compare_with_base_head() {
        let cli = Cli::parse_from([
            "cov-info", "--token", "t", "compare", "--base", "abc", "--head", "def",
        ]);
        if let Command::Compare { base, head, pullid } = cli.command {
            assert_eq!(base.as_deref(), Some("abc"));
            assert_eq!(head.as_deref(), Some("def"));
            assert!(pullid.is_none());
        } else {
            panic!("expected Compare");
        }
    }

    #[test]
    fn parse_file_report() {
        let cli = Cli::parse_from([
            "cov-info",
            "--token",
            "t",
            "file-report",
            "src/main.rs",
            "--branch",
            "main",
        ]);
        if let Command::FileReport { path, branch, .. } = cli.command {
            assert_eq!(path, "src/main.rs");
            assert_eq!(branch.as_deref(), Some("main"));
        } else {
            panic!("expected FileReport");
        }
    }

    #[test]
    fn parse_format_text() {
        let cli = Cli::parse_from(["cov-info", "--token", "t", "--format", "text", "components"]);
        assert!(matches!(cli.format, OutputFormat::Text));
    }

    #[test]
    fn service_api_strings() {
        assert_eq!(Service::Github.as_api_str(), "github");
        assert_eq!(Service::GithubEnterprise.as_api_str(), "github_enterprise");
        assert_eq!(Service::BitbucketServer.as_api_str(), "bitbucket_server");
    }
}
