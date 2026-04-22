# mago-twig-syntax

A lossless lexer, AST, and parser for [Twig 3](https://twig.symfony.com/) templates.

The crate is purely syntactic. It does not resolve filters, functions, or
tests, it does not load templates, and unknown tag names are accepted as
opaque `Statement::Unknown` nodes. It is designed to back tooling - linters,
formatters, language servers, static analysers - rather than a runtime.

## Design

- **Lossless.** Every byte of the input appears in the value of exactly one
  emitted token. Trivia (whitespace, `{# … #}` comments, inline `# …`
  comments) is collected on a side channel and attached to the [`Template`]
  so the source can be reconstructed byte-for-byte from the AST.
- **Arena-allocated AST.** Every node lives in a caller-supplied
  [`bumpalo::Bump`]. No reference counting, no per-node `Drop`.
- **Recursive-descent parser.** Precedence-climbing for expressions,
  with a shared `parse_comma_separated_sequence` helper for delimited lists.
- **Error recovery.** Top-level parsing never bails: every [`Template`]
  carries the parsed statements _and_ a list of [`ParseError`] for the
  parts that did not parse.

## Adding it to your project

```toml
[dependencies]
mago-database = "1"
mago-twig-syntax = "1"
bumpalo = "3"
```

## Usage

```rust
use bumpalo::Bump;
use mago_database::file::FileId;
use mago_twig_syntax::parser::parse_file_content;

let arena = Bump::new();
let template = parse_file_content(&arena, FileId::zero(), "Hello, {{ name }}!");

assert!(!template.has_errors());

for statement in &template.statements {
    // ...
}
```

For a real source file:

```rust
use mago_twig_syntax::parser::parse_file;

let arena = Bump::new();
let template = parse_file(&arena, &file);
```

For pre-built lexer/parser wiring, see [`parser::Parser::new`] and
[`parser::Parser::from_lexer`].

## Performance

On Apple Silicon (release profile) the crate sustains roughly:

| stage  | small (~1 KB) | medium (~24 KB) | large (~30 KB) | huge (~85 KB) |
| ------ | ------------- | --------------- | -------------- | ------------- |
| lexer  | 290 MiB/s     | 315 MiB/s       | 280 MiB/s      | 270 MiB/s     |
| parser | 113 MiB/s     | 141 MiB/s       | 134 MiB/s      | 132 MiB/s     |

Run `cargo bench -p mago-twig-syntax` to reproduce.

## AST traversal

A macro-generated [`MutWalker`] / [`Walker`] trait pair is provided in
[`walker`], with `walk_in_*` / `walk_out_*` hooks per node type. For one-off
inspection, [`ast::Node`] is a non-exhaustive enum covering every node kind.

## License

Dual-licensed under the same terms as the rest of the [Mago] workspace.

[`Template`]: crate::ast::Template
[`ParseError`]: crate::error::ParseError
[`MutWalker`]: crate::walker::MutWalker
[`Walker`]: crate::walker::Walker
[`bumpalo::Bump`]: https://docs.rs/bumpalo
[Mago]: https://github.com/carthage-software/mago
