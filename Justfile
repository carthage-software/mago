default:
  @just --list

# build the library
build:
    cargo build --release

# build the webassembly module
build-wasm:
    cd crates/wasm && wasm-pack build --release --out-dir pkg

# detect linting problems.
lint:
    cargo +nightly fmt --all -- --check --unstable-features
    cargo +nightly clippy --workspace --all-targets --all-features -- -D warnings
    cargo +nightly check --workspace --locked

# fix linting problems.
fix:
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
    cargo fix --allow-dirty --allow-staged
    cargo fmt --all -- --unstable-features

# run tests
test:
    cargo test --workspace --locked --all-targets

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
    cargo publish -p mago-traverser --allow-dirty
    cargo publish -p mago-ast-utils --allow-dirty
    cargo publish -p mago-composer --allow-dirty
    cargo publish -p mago-docblock --allow-dirty
    cargo publish -p mago-lexer --allow-dirty
    cargo publish -p mago-parser --allow-dirty
    cargo publish -p mago-formatter --allow-dirty
    cargo publish -p mago-names --allow-dirty
    cargo publish -p mago-symbol-table --allow-dirty
    cargo publish -p mago-semantics --allow-dirty
    cargo publish -p mago-reference --allow-dirty
    cargo publish -p mago-typing --allow-dirty
    cargo publish -p mago-reflector --allow-dirty
    cargo publish -p mago-linter --allow-dirty
    cargo publish -p mago-wasm --allow-dirty
    cargo publish --allow-dirty
