# Fennec: The Oxidized PHP Toolchain

Fennec is a toolchain for PHP that aims to provide a set of tools to help developers write better code.
It is inspired by the Rust programming language and its toolchain, and aims to provide similar functionality for PHP.

## Disclaimer

> [!WARNING]
> Fennec is in an early stage of development. Many features are not yet implemented, and existing functionality may change, break, or stop working without notice.
> While we are not actively promoting or advertising the project, we are working in public to share our progress with the community.

## Roadmap

### Core Functionality

- [x] String Interning: [`crates/interner`](crates/interner)
- [x] Lexer: [`crates/lexer`](crates/lexer) [`crates/token`](crates/token)
- [x] AST: [`crates/ast`](crates/ast)
- [x] Parser: [`crates/parser`](crates/parser)
- [x] Source Management: [`crates/source`](crates/source)
- [x] AST Traversal / Walk: [`crates/traverser`](crates/traverser) [`crates/walker`](crates/walker)
- [x] Name Resolution: [`crates/names`](crates/names)
- [x] Code Fixer: [`crates/fixer`](crates/fixer)
- [x] Error Reporting: [`crates/reporting`](crates/reporting)
- [x] Semantic Analysis: [`crates/semantics`](crates/semantics)
- [x] Symbol Table: [`crates/symbol-table`](crates/symbol-table)
- [x] Linter: [`crates/linter`](crates/linter)
- [x] Configuration: [`crates/config`](crates/config)
- [x] String Case Conversion: [`crates/casing`](crates/casing)
- [x] Reflections: [`crates/reflection`](crates/reflection)
- [x] Reflector: [`crates/reflector`](crates/reflector), [`crates/scanner`](crates/scanner)
- [x] Type Inference: [`crates/inference`](crates/inference)
- [ ] Formatter
- [ ] Static Analyzer
- [ ] Refactoring
- [ ] Code Generation
- [ ] Documentation Generation
- [x] Docblock Parser [`crates/docblock`](crates/docblock)
- [ ] Test Runner

### Tooling

- [ ] CLI Tool: [`src/main.rs`](src/main.rs) - In Progress, Basic Functionality Implemented.
- [ ] Web Interface
- [ ] Language Server Protocol
- [ ] Editor Integration

## Installation

```bash
cargo install --git https://github.com/carthage-software/fennec
```

## Installation from source

```bash
git clone https://github.com/carthage-software/fennec
cd fennec
cargo install --path .
```

## Usage

For a quick start, you can refer to the example configuration files provided:

- Simple configuration: [`examples/fennec.toml`](examples/fennec.toml)
- Full configuration with all possible options: [`examples/fennec-full.toml`](examples/fennec-full.toml)

You can try Fennec by navigating to the [`examples`](examples) directory and running the linter on the sample PHP files:

```bash
cd examples
fennec lint
```

This will analyze the PHP files located in the [`examples/src/`](examples/src) directory and display any linting errors.

## How You Can Help

Fennec is a community-driven project, and we’d love for you to join us! Here are some ways you can contribute:

- _Suggest Ideas_: Have an idea for Fennec? We’re open to suggestions that can make the toolchain even better!
- _Help Write Documentation_: Clear, user-friendly documentation is key to making Fennec accessible to everyone. If you enjoy writing or organizing docs, we'd love your help.
- _Contribute Code_: Join us in building Fennec! Please discuss any feature or bug fixes in the issues first to ensure we coordinate effectively.
- _Sponsor the Project_: If you’d like to support Fennec financially, consider sponsoring [@azjezz](https://github.com/azjezz). Every contribution helps!
- _Help with Art_: Fennec could use a logo! We’d appreciate the help of a skilled artist to create an original logo for Fennec. (Please note that AI-generated art will not be accepted.)

## Inspiration

Fennec is inspired by several tools and projects that have significantly contributed to the development community:

- [Clippy](https://github.com/rust-lang/rust-clippy): A collection of lints to catch common mistakes and improve your Rust code.
- [OXC](https://github.com/oxc-project/oxc/): A JavaScript toolchain written in Rust.
- [php-rust-tools/parser](https://github.com/php-rust-tools/parser/): A PHP parser written in Rust, which influenced our parsing approach.
- [slackhq/hakana](https://github.com/slackhq/hakana/): A static analysis tool for HackLang written in Rust, by the creator of [Psalm](https://github.com/vimeo/psalm).

These tools have inspired us and helped shape Fennec's design and functionality.

## Acknowledgements

We would like to acknowledge the following PHP tools that have greatly helped hundreds of thousands of PHP developers in their journey,
ourselves included:

- [PHP CS Fixer](https://github.com/PHP-CS-Fixer/PHP-CS-Fixer): A tool to automatically fix PHP Coding Standards issues.
- [Psalm](https://github.com/vimeo/psalm): A static analysis tool for finding errors in PHP applications.
- [PHPStan](https://github.com/phpstan/phpstan): PHP Static Analysis Tool.
- [PHP_CodeSniffer](https://github.com/squizlabs/PHP_CodeSniffer): Detects violations of a defined set of coding standards.

While Fennec is intended to be a comprehensive toolchain that may eventually replace some of these tools,
we deeply appreciate their contributions and the foundation they have built for the PHP community.

## License

Fennec is licensed under either of

- MIT License (MIT) - see [LICENSE-MIT](./LICENSE-MIT) file for details
- Apache License, Version 2.0 (Apache-2.0) - see [LICENSE-APACHE](./LICENSE-APACHE) file for details

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Fennec by you shall be dual licensed as above, without any additional terms or conditions.

---

Thank you for your interest in Fennec. We look forward to sharing our progress and collaborating with the community as the project evolves.
