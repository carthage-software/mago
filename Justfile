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
    cargo publish -p mago-casing --allow-dirty
    cargo publish -p mago-php-version --allow-dirty
    cargo publish -p mago-fixer --allow-dirty
    cargo publish -p mago-trinary --allow-dirty
    cargo publish -p mago-interner --allow-dirty
    cargo publish -p mago-source --allow-dirty
    cargo publish -p mago-span --allow-dirty
    cargo publish -p mago-reporting --allow-dirty
    cargo publish -p mago-reflection --allow-dirty
    cargo publish -p mago-token --allow-dirty
    cargo publish -p mago-ast --allow-dirty
    cargo publish -p mago-walker --allow-dirty
    cargo publish -p mago-ast-utils --allow-dirty
    cargo publish -p mago-composer --allow-dirty
    cargo publish -p mago-docblock --allow-dirty
    cargo publish -p mago-lexer --allow-dirty
    cargo publish -p mago-parser --allow-dirty
    cargo publish -p mago-formatter --allow-dirty
    cargo publish -p mago-names --allow-dirty
    cargo publish -p mago-reference --allow-dirty
    cargo publish -p mago-typing --allow-dirty
    cargo publish -p mago-project --allow-dirty
    cargo publish -p mago-linter --allow-dirty
    cargo publish -p mago-wasm --allow-dirty
    cargo publish --allow-dirty

# Cleans all build artifacts from the workspace.
clean:
    cargo clean --workspace

# Regenerates PHPStorm stubs files from the JetBrains phpstorm-stubs repository.
stubs:
    git clone https://github.com/JetBrains/phpstorm-stubs {{template_dir}}
    find {{template_dir}}/* -maxdepth 0 -type d -exec cp -r {} stubs \;
    cp {{template_dir}}/LICENSE stubs/
    rm -rf stubs/tests
    find stubs -name ".phpstorm.meta.php" -delete
    rm -rf {{template_dir}}
