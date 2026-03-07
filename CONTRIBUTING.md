# Contributing to codecov-query

Contributions of all kinds are welcome — bug reports, new features, documentation improvements, and code changes.

---

## Getting started

```sh
git clone https://github.com/mpecan/codecov-query
cd codecov-query
git config core.hooksPath .githooks
cargo build
cargo test
```

The `git config` line enables the shared pre-commit hook that runs `cargo fmt --check` and `cargo clippy` before each commit.

The project requires Rust 1.93.0 or later. See `rust-toolchain.toml` for the pinned version.

---

## Project structure

| Path | Description |
|---|---|
| `src/main.rs` | CLI entry point, owner/repo resolution, subcommand dispatch |
| `src/cli.rs` | Clap derive structs, subcommands, and argument definitions |
| `src/client.rs` | `CodecovClient`: URL building, HTTP requests, response handling |
| `src/models.rs` | Serde response types (`Totals`, `Repo`, `Commit`, `Pull`, etc.) |
| `src/output.rs` | JSON and human-readable text formatters |
| `src/git.rs` | Auto-detect owner/repo from git remote origin |

---

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `ci`, `perf`

Keep commits atomic — one logical change per commit.

---

## Code quality

Before opening a PR:

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

All three must pass clean. CI runs the same checks.

### Lint rules

The project enforces Clippy's `pedantic` and `nursery` lint groups as warnings (see `Cargo.toml`).

### Limits

- **Functions:** stay under 60 lines (Clippy enforces this)
- **Line width:** 100 characters (rustfmt enforces this)

---

## Pull requests

- Target the `main` branch
- Include tests for any changed behaviour
- Keep PRs focused — one feature or fix per PR
- Reference relevant issues in the PR description (`Closes: #N`)

---

## License

By contributing you agree that your changes will be licensed under the [MIT License](LICENSE).
