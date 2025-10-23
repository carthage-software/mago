---
title: Command-Line Interface
---

# Command-Line Interface

This page provides an overview of the Mago command-line interface (CLI), including global options and subcommands.

## Usage Pattern

Mago follows a standard `[GLOBAL OPTIONS] <SUBCOMMAND>` pattern. Global options must be specified **before** the subcommand.

```sh
# Correct: Global option before subcommand
mago --colors=never lint

# Incorrect: Global option after subcommand
mago lint --colors=never
```

## Global Options

These options can be used with the main `mago` command and any of its subcommands. They control the overall execution environment and configuration.

| Flag, Alias(es)                   | Description                                                                                                     |
| :-------------------------------- | :-------------------------------------------------------------------------------------------------------------- |
| `--workspace <PATH>`              | Sets the path to the workspace directory, which is the root of your project. Defaults to the current directory. |
| `--config <PATH>`                 | Specifies the path to the configuration file. If not provided, Mago searches for `mago.toml` in the workspace.  |
| `--php-version <VERSION>`         | Overrides the PHP version (e.g., `8.2`) specified in the configuration file.                                    |
| `--threads <NUMBER>`              | Overrides the number of threads Mago will use. Defaults to the number of available logical CPUs.                |
| `--allow-unsupported-php-version` | Allows Mago to run against a PHP version that is not officially supported. Use with caution.                    |
| `--no-color`, `--no-colors`       | Disables all color in the output. This option has been deprecated. Use `--colors never` instead.                |
| `--colors <WHEN>`                 | Controls when to use colors in the output. Options: `always`, `never`, `auto` (default).                        |
| `-h`, `--help`                    | Print help information.                                                                                         |
| `-V`, `--version`                 | Print version information.                                                                                      |

## Subcommands

Mago is organized into several tools and utility commands, each accessed via a subcommand.

### Tools

| Command                                                | Description                                                |
| :----------------------------------------------------- | :--------------------------------------------------------- |
| [`mago analyze`](/tools/analyzer/command-reference.md) | Analyzes PHP code for type-safety and other issues.        |
| [`mago ast`](/tools/lexer-parser/command-reference.md) | Inspects the Abstract Syntax Tree of a PHP file.           |
| [`mago format`](/tools/formatter/command-reference.md) | Formats PHP code.                                          |
| [`mago guard`](/tools/guard/command-reference.md)      | Enforces architectural rules and boundaries.               |
| [`mago lint`](/tools/linter/command-reference.md)      | Lints PHP code for style, correctness, and best practices. |

### Utility Commands

| Command                                                    | Description                                             |
| :--------------------------------------------------------- | :------------------------------------------------------ |
| [`mago config`](/guide/configuration)                      | Displays the final, merged configuration Mago is using. |
| [`mago list-files`](/guide/list-files)                     | Displays a list of the files scanned by Mago.           |
| [`mago generate-completions`](/guide/generate-completions) | Generate shell completions for Mago.                    |
| [`mago init`](/guide/initialization)                       | Initializes a new Mago configuration file.              |
| [`mago self-update`](/guide/upgrading)                     | Updates Mago to the latest version.                     |
