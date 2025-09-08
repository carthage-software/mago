template_dir := `mktemp -d`

# Lists all available commands.
list:
    @just --list

# Builds the library in release mode.
build:
    cargo build --release

# Builds the webassembly module.
build-wasm:
    cd crates/wasm && wasm-pack build --release --out-dir pkg

# Detects linting problems using rustfmt, clippy, and cargo check.
lint:
    cargo +nightly fmt --all -- --check --unstable-features
    cargo +nightly clippy --workspace --all-targets --all-features -- -D warnings
    cargo +nightly check --workspace --locked

# Fixes linting problems automatically using clippy, cargo fix, and rustfmt.
fix:
    cargo +nightly clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
    cargo +nightly fix --allow-dirty --allow-staged
    cargo +nightly fmt --all -- --unstable-features

# Runs all tests in the workspace.
test:
    cargo test --workspace --locked --all-targets

# Publishes all crates to crates.io in the correct order.
publish:
    # Note: the order of publishing is important, as some crates depend on others.
    cargo publish -p mago-pager
    cargo publish -p mago-casing
    cargo publish -p mago-php-version
    cargo publish -p mago-fixer
    cargo publish -p mago-atom
    cargo publish -p mago-database
    cargo publish -p mago-span
    cargo publish -p mago-reporting
    cargo publish -p mago-syntax-core
    cargo publish -p mago-syntax
    cargo publish -p mago-collector
    cargo publish -p mago-type-syntax
    cargo publish -p mago-composer
    cargo publish -p mago-docblock
    cargo publish -p mago-formatter
    cargo publish -p mago-names
    cargo publish -p mago-semantics
    cargo publish -p mago-codex
    cargo publish -p mago-prelude
    cargo publish -p mago-algebra
    cargo publish -p mago-analyzer
    cargo publish -p mago-linter
    cargo publish -p mago-wasm
    cargo publish

# Cleans all build artifacts from the workspace.
clean:
    cargo clean --workspace
