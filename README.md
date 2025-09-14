<p align="center">
  <img src="docs/public/assets/banner.svg" alt="Mago Banner" width="600" />
</p>

<div align="center">

**An extremely fast PHP linter, formatter, and static analyzer, written in Rust.**

</div>

<div align="center">

[![CI Status](https://github.com/carthage-software/mago/actions/workflows/ci.yml/badge.svg)](https://github.com/carthage-software/mago/actions/workflows/ci.yml)
[![CD Status](https://github.com/carthage-software/mago/actions/workflows/cd.yml/badge.svg)](https://github.com/carthage-software/mago/actions/workflows/cd.yml)
[![Crates.io](https://img.shields.io/crates/v/mago.svg)](https://crates.io/crates/mago)
[![Latest Stable Version for PHP](https://poser.pugx.org/carthage-software/mago/v)](https://packagist.org/packages/carthage-software/mago)
[![Latest Unstable Version for PHP](https://poser.pugx.org/carthage-software/mago/v/unstable)](https://packagist.org/packages/carthage-software/mago)
[![Total Composer Downloads](http://poser.pugx.org/carthage-software/mago/downloads)](https://packagist.org/packages/carthage-software/mago)
[![License](https://img.shields.io/crates/l/mago.svg)](https://github.com/carthage-software/mago/blob/main/LICENSE-MIT)

</div>

**Mago** is a comprehensive toolchain for PHP that helps developers write better code. Inspired by the Rust ecosystem, Mago brings speed, reliability, and an exceptional developer experience to PHP projects of all sizes.

## Table of Contents

- [Installation](#installation)
- [Getting Started](#getting-started)
- [Features](#features)
- [Our Sponsors](#our-sponsors)
- [Contributing](#contributing)
- [Inspiration & Acknowledgements](#inspiration--acknowledgements)
- [License](#license)

## Installation

The most common way to install Mago on macOS and Linux is by using our shell script:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash
```

For all other installation methods, including Homebrew, Composer, and Cargo, please refer to our official **[Installation Guide](https://mago.carthage.software/guide/installation)**.

## Getting Started

To get started with Mago and learn how to configure your project, please visit our **[Getting Started Guide](https://mago.carthage.software/guide/getting-started)** in the official documentation.

## Features

- ⚡️ Extremely Fast: Built in Rust for maximum performance.
- 🔍 Lint: Identify issues in your codebase with customizable rules.
- 🔬 Static Analysis: Perform deep analysis of your codebase to catch potential type errors and bugs.
- 🛠️ Automated Fixes: Apply fixes for many lint issues automatically.
- 📜 Formatting: Automatically format your code to adhere to best practices and style guides.
- 🧠 Semantic Checks: Ensure code correctness with robust semantic analysis.
- 🌳 AST Visualization: Explore your code’s structure with Abstract Syntax Tree (AST) parsing.

<!-- START-SPONSORS -->
## Our Sponsors

<p align="center"><a href="https://github.com/jasonrm" title="Jason R. McNeil"><kbd><img src="https://avatars.githubusercontent.com/u/39949?u=69c0e4fb08c439250978d41dbc3371d2f0609b98&v=4&s=160" width="80" height="80" alt="Jason R. McNeil" /></kbd></a><a href="https://github.com/vvvinceocam" title="Vincent Berset"><kbd><img src="https://avatars.githubusercontent.com/u/5173120?u=95efc76cd8fc804536dc6dd25781a95b650bf902&v=4&s=160" width="80" height="80" alt="Vincent Berset" /></kbd></a></p><p align="center"><a href="https://github.com/TicketSwap" title="TicketSwap"><kbd><img src="https://avatars.githubusercontent.com/u/5766233?v=4&s=120" width="60" height="60" alt="TicketSwap" /></kbd></a></p>

[See all sponsors](SPONSORS.md)
<!-- END-SPONSORS -->

## Contributing

Mago is a community-driven project, and we welcome contributions! Whether you're reporting bugs, suggesting features, writing documentation, or submitting code, your help is valued.

- See our [Contributing Guide](./CONTRIBUTING.md) to get started.
- Join the discussion on [Discord](https://discord.gg/mwyyjr27eu).

## Inspiration & Acknowledgements

Mago stands on the shoulders of giants. Our design and functionality are heavily inspired by pioneering tools in both the Rust and PHP ecosystems.

### Inspirations:

- [Clippy](https://github.com/rust-lang/rust-clippy): For its comprehensive linting approach.
- [OXC](https://github.com/oxc-project/oxc/): A major inspiration for building a high-performance toolchain in Rust.
- [Hakana](https://github.com/slackhq/hakana/): For its deep static analysis capabilities.

### Acknowledgements:

We deeply respect the foundational work of tools like [PHP-CS-Fixer](https://github.com/PHP-CS-Fixer/PHP-CS-Fixer), [Psalm](https://github.com/vimeo/psalm), [PHPStan](https://github.com/phpstan/phpstan), and [PHP_CodeSniffer](https://github.com/PHPCSStandards/PHP_CodeSniffer). While Mago aims to offer a unified and faster alternative, these tools paved the way for modern PHP development.

## License

Mago is dual-licensed under your choice of the following:

- MIT License ([LICENSE-MIT](./LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE))
