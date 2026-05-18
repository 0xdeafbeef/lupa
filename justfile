# Display available recipes.
default:
    @just --list

# Format the workspace with the project rustfmt settings.
fmt:
    cargo +nightly fmt

# Check formatting with the project rustfmt settings.
fmt-check:
    cargo +nightly fmt --check

# Run strict Clippy checks.
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Run the workspace test suite.
test:
    cargo test --workspace

# Run all local verification gates.
check: fmt-check clippy test
