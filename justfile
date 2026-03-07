# codecov-query development tasks
# Run `just` to see all available recipes

# List available recipes
default:
    @just --list

# Set up the development environment (git hooks)
setup:
    git config core.hooksPath .githooks
    @echo "Git hooks enabled."

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run all tests
test:
    cargo test --all-targets

# Run a specific test by name
test-one name:
    cargo test {{name}}

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Fix formatting
fmt:
    cargo fmt --all

# Run clippy lints
clippy:
    cargo clippy --all-targets -- -D warnings

# Run all checks (fmt, clippy, test) — same as CI
check: fmt-check clippy test

# Fix formatting then run all checks
fix-and-check: fmt clippy test

# Run the CLI with arguments (e.g. `just run pr-summary 107`)
run *args:
    cargo run -- {{args}}

# Run the CLI against a specific owner/repo (e.g. `just query mpecan wizzard pr-summary 107`)
query owner repo *args:
    cargo run -- --owner {{owner}} --repo {{repo}} {{args}}

# Clean build artifacts
clean:
    cargo clean

# Watch for changes and run tests (requires cargo-watch)
watch:
    cargo watch -x 'test --all-targets'

# Generate and open documentation
doc:
    cargo doc --open
