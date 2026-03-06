mod cli;
mod client;
mod git;
mod models;
mod output;

use anyhow::{Context, Result};
use clap::Parser;

use cli::{Cli, Command, OutputFormat};
use client::CodecovClient;
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
        Command::Compare { base, head, pullid } => {
            let result = client.compare(base.as_deref(), head.as_deref(), *pullid)?;
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
