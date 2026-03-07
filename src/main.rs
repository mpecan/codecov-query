mod cli;
mod client;
mod git;
mod models;
mod output;

use anyhow::{Context, Result};
use clap::Parser;

use cli::{Cli, Command, OutputFormat};
use client::CodecovClient;
use models::{ChangedFileSummary, PrSummary};
use output::{TextFormat, print_json};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let token = cli
        .token
        .as_deref()
        .context("--token or CODECOV_TOKEN env var is required")?;

    let (detected_owner, detected_repo) = git::detect_owner_repo()
        .map(|(o, r)| (Some(o), Some(r)))
        .unwrap_or_default();

    let owner = cli
        .owner
        .as_deref()
        .or(detected_owner.as_deref())
        .context("--owner is required (could not auto-detect from git remote)")?;

    let needs_repo = !matches!(cli.command, Command::Repos { .. });
    let repo = cli.repo.as_deref().or(detected_repo.as_deref());
    if needs_repo && repo.is_none() {
        anyhow::bail!("--repo is required (could not auto-detect from git remote)");
    }

    let service = cli.service.as_api_str();
    let client = CodecovClient::new(token, service, owner, repo)?;

    run_command(&client, &cli.command, &cli.format)
}

#[allow(clippy::too_many_lines)]
fn run_command(client: &CodecovClient, command: &Command, format: &OutputFormat) -> Result<()> {
    match command {
        Command::Repos {
            active,
            search,
            names,
            page,
            page_size,
        } => {
            let result = client.list_repos(
                *active,
                search.as_deref(),
                names.as_deref(),
                *page,
                *page_size,
            )?;
            output_result(format, &result)
        }
        Command::Repo => output_result(format, &client.get_repo()?),
        Command::Totals {
            sha,
            branch,
            path,
            flag,
            component_id,
        } => {
            let result = client.get_totals(
                sha.as_deref(),
                branch.as_deref(),
                path.as_deref(),
                flag.as_deref(),
                component_id.as_deref(),
            )?;
            output_result(format, &result)
        }
        Command::Commits {
            branch,
            page,
            page_size,
        } => {
            let result = client.list_commits(branch.as_deref(), *page, *page_size)?;
            output_result(format, &result)
        }
        Command::Commit { commitid } => output_result(format, &client.get_commit(commitid)?),
        Command::Branches { page, page_size } => {
            output_result(format, &client.list_branches(*page, *page_size)?)
        }
        Command::Branch { name } => output_result(format, &client.get_branch(name)?),
        Command::Pulls {
            state,
            start_date,
            ordering,
            page,
            page_size,
        } => {
            let result = client.list_pulls(
                state.as_deref(),
                start_date.as_deref(),
                ordering.as_deref(),
                *page,
                *page_size,
            )?;
            output_result(format, &result)
        }
        Command::Pull { pullid } => output_result(format, &client.get_pull(*pullid)?),
        Command::Compare {
            base,
            head,
            pullid,
            summary,
        } => {
            let result = client.compare(base.as_deref(), head.as_deref(), *pullid)?;
            if *summary {
                let result = result.into_summary();
                output_result(format, &result)
            } else {
                output_result(format, &result)
            }
        }
        Command::PrSummary { pullid } => {
            let result = build_pr_summary(client, *pullid)?;
            output_result(format, &result)
        }
        Command::FileReport { path, sha, branch } => {
            let result = client.get_file_report(path, sha.as_deref(), branch.as_deref())?;
            output_result(format, &result)
        }
        Command::Flags { page, page_size } => {
            output_result(format, &client.list_flags(*page, *page_size)?)
        }
        Command::Components => output_result(format, &client.list_components()?),
    }
}

fn output_result<T: serde::Serialize + TextFormat>(format: &OutputFormat, value: &T) -> Result<()> {
    match format {
        OutputFormat::Json => print_json(value),
        OutputFormat::Text => {
            value.print_text();
            Ok(())
        }
    }
}

fn build_pr_summary(client: &CodecovClient, pullid: u64) -> Result<PrSummary> {
    let pull = client.get_pull(pullid)?;
    let comparison = client.compare(None, None, Some(pullid))?;

    let base_coverage = comparison
        .totals
        .as_ref()
        .and_then(|t| t.base.as_ref())
        .and_then(|b| b.coverage);
    let head_coverage = comparison
        .totals
        .as_ref()
        .and_then(|t| t.head.as_ref())
        .and_then(|h| h.coverage);
    let coverage_delta = match (base_coverage, head_coverage) {
        (Some(b), Some(h)) => Some(h - b),
        _ => None,
    };
    let patch_totals = comparison.totals.as_ref().and_then(|t| t.patch.as_ref());
    let patch_coverage = patch_totals.and_then(|p| p.coverage);
    let patch_hits = patch_totals.and_then(|p| p.hits);
    let patch_lines = patch_totals.and_then(|p| p.lines);

    let status = match coverage_delta {
        Some(d) if d < 0.0 => "COVERAGE DECREASED",
        Some(d) if d > 0.0 => "IMPROVED",
        Some(_) => "UNCHANGED",
        None => "UNKNOWN",
    }
    .to_string();

    let changed_files = comparison
        .files
        .as_ref()
        .map(|files| {
            files
                .iter()
                .filter(|f| f.has_diff == Some(true))
                .map(comparison_file_to_summary)
                .collect()
        })
        .unwrap_or_default();

    Ok(PrSummary {
        pullid,
        title: pull.title,
        state: pull.state,
        ci_passed: pull.ci_passed,
        base_coverage,
        head_coverage,
        coverage_delta,
        patch_coverage,
        patch_hits,
        patch_lines,
        status,
        changed_files,
    })
}

fn comparison_file_to_summary(f: &models::ComparisonFile) -> ChangedFileSummary {
    let path = f
        .name
        .as_ref()
        .and_then(|n| n.head.as_deref().or(n.base.as_deref()))
        .unwrap_or("unknown")
        .to_string();

    let file_patch = f.totals.as_ref().and_then(|t| t.patch.as_ref());
    let file_patch_coverage = file_patch.and_then(|p| p.coverage);
    let file_patch_hits = file_patch.and_then(|p| p.hits);
    let file_patch_lines = file_patch.and_then(|p| p.lines);
    let file_patch_misses = file_patch.and_then(|p| p.misses);

    let file_base_coverage = f
        .totals
        .as_ref()
        .and_then(|t| t.base.as_ref())
        .and_then(|b| b.coverage);
    let file_head_coverage = f
        .totals
        .as_ref()
        .and_then(|t| t.head.as_ref())
        .and_then(|h| h.coverage);

    let uncovered_lines = f
        .lines
        .as_ref()
        .map(|l| models::extract_uncovered_lines(l, true))
        .unwrap_or_default();

    let file_status = match file_patch_coverage {
        Some(c) if c >= 50.0 => "OK",
        Some(_) => "LOW COVERAGE",
        None => "NO COVERAGE DATA",
    }
    .to_string();

    ChangedFileSummary {
        path,
        patch_coverage: file_patch_coverage,
        patch_hits: file_patch_hits,
        patch_lines: file_patch_lines,
        patch_misses: file_patch_misses,
        base_coverage: file_base_coverage,
        head_coverage: file_head_coverage,
        status: file_status,
        uncovered_lines,
    }
}
