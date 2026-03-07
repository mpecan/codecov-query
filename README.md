# codecov-query

A CLI tool to query the [Codecov API v2](https://docs.codecov.com/reference). Designed for both human users and LLM-driven workflows.

Unlike the official [codecov-cli](https://github.com/codecov/codecov-cli) (which focuses on *uploading* coverage reports), `codecov-query` is a **read-only** tool for retrieving coverage data — totals, commits, branches, pull requests, comparisons, file reports, flags, and components.

## Installation

### From crates.io

```sh
cargo install codecov-query
```

### From source

```sh
git clone https://github.com/mpecan/codecov-query
cd codecov-query
cargo install --path .
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/mpecan/codecov-query/releases).

## Authentication

A Codecov API token is required. Provide it via:

- `--token <TOKEN>` flag, or
- `CODECOV_TOKEN` environment variable

Generate a token at [codecov.io/account](https://app.codecov.io/account).

## Usage

### Auto-detection

When run inside a git repository, `codecov-query` automatically detects the owner and repo from the `origin` remote. You can override with `--owner` / `--repo`.

### Output formats

- `--format text` (default) — human-readable summary, compact and agent-friendly
- `--format json` — structured JSON, ideal for piping to `jq` or feeding to LLMs

### Examples

```sh
# Coverage totals (auto-detect repo from git remote)
codecov-query totals
codecov-query totals --branch main --format text

# Explicit owner/repo
codecov-query --owner mpecan --repo myproject totals

# List repositories
codecov-query --owner mpecan repos --active true

# List commits
codecov-query commits --branch main --page-size 5

# Get a specific commit
codecov-query commit abc123def

# Branches
codecov-query branches
codecov-query branch main

# Pull requests
codecov-query pulls --state open
codecov-query pull 42

# Compare coverage
codecov-query compare --pullid 42
codecov-query compare --pullid 42 --summary   # filtered to changed files only
codecov-query compare --base abc123 --head def456

# PR coverage summary (combines pull + compare into one view)
codecov-query pr-summary 42

# File-level coverage
codecov-query file-report src/main.rs --branch main

# Flags and components
codecov-query flags
codecov-query components

# Pipe to jq (use --format json for structured output)
codecov-query totals --format json | jq '.totals.coverage'
```

## Subcommands

| Subcommand | Description | Key arguments |
|---|---|---|
| `repos` | List repositories for an owner | `--active`, `--search`, `--names` |
| `repo` | Get repository details | — |
| `totals` | Coverage totals | `--sha`, `--branch`, `--path`, `--flag`, `--component-id` |
| `commits` | List commits | `--branch` |
| `commit` | Get a specific commit | positional `commitid` |
| `branches` | List branches | — |
| `branch` | Get a specific branch | positional `name` |
| `pulls` | List pull requests | `--state`, `--start-date`, `--ordering` |
| `pull` | Get a specific PR | positional `pullid` |
| `compare` | Compare coverage | `--base` + `--head` OR `--pullid`, `--summary` |
| `pr-summary` | PR coverage summary with uncovered line ranges | positional `pullid` |
| `file-report` | File-level coverage | positional `path`, `--sha`, `--branch` |
| `flags` | List flags | — |
| `components` | List components | — |

All list subcommands support `--page` and `--page-size` for pagination.

## Global options

| Option | Description | Default |
|---|---|---|
| `--token` | Codecov API token (or `CODECOV_TOKEN` env) | required |
| `--service` | Git hosting service | `github` |
| `--owner` / `-o` | Repository owner | auto-detected |
| `--repo` / `-r` | Repository name | auto-detected |
| `--format` | Output format (`json` or `text`) | `text` |

Supported services: `github`, `gitlab`, `bitbucket`, `github-enterprise`, `gitlab-enterprise`, `bitbucket-server`.

## License

[MIT](LICENSE)
