+++
title = "Command-line interface"
description = "Global options, subcommands, environment variables, and exit codes."
nav_order = 10
nav_section = "Fundamentals"
+++
# Command-line interface

Every Mago invocation follows the pattern `mago [GLOBAL OPTIONS] <SUBCOMMAND>`. Global options must come before the subcommand.

```sh
mago --colors=never lint        # correct
mago lint --colors=never        # wrong, --colors is a global option
```

## Global options

These options apply to every subcommand and control the runtime, configuration discovery, and output.

| Flag | Description |
| :--- | :--- |
| `--workspace <PATH>` | Workspace root. Defaults to the current directory. |
| `--config <PATH>` | Path to the config file. Without it, Mago searches the workspace, `$XDG_CONFIG_HOME`, `~/.config`, and `~`. See [discovery](/guide/configuration/#discovery). |
| `--php-version <VERSION>` | Override the configured PHP version, e.g. `8.2`. |
| `--threads <NUMBER>` | Override the thread count. Defaults to the number of logical CPUs. |
| `--allow-unsupported-php-version` | Run against a PHP version Mago does not officially support. Use with care. |
| `--no-version-check` | Silence the warning emitted on minor or patch drift from the project's pinned version. Major drift remains fatal. See [version pinning](/guide/configuration/#version-pinning). |
| `--colors <WHEN>` | When to colour output: `always`, `never`, or `auto` (default). |
| `-h`, `--help` | Print help and exit. |
| `-V`, `--version` | Print the installed version and exit. |

## Environment variables

Most configuration overrides use the `MAGO_*` prefix and are documented on the [environment variables page](/guide/environment-variables/). The two you are most likely to set day-to-day are:

| Variable | Purpose |
| :--- | :--- |
| `MAGO_LOG` | Log filter for tracing output. Values: `trace`, `debug`, `info`, `warn`, `error`. |
| `MAGO_EDITOR_URL` | URL template for clickable file paths in terminal output. See [editor integration](/guide/configuration/#editor-integration). |

## Subcommands

The core tools:

| Command | Description |
| :--- | :--- |
| [`mago analyze`](/tools/analyzer/command-reference/) | Static analysis: type errors, logic bugs. |
| [`mago ast`](/guide/inspecting-the-ast/) | Print the AST of a PHP file. |
| [`mago format`](/tools/formatter/command-reference/) | Format PHP files. |
| [`mago guard`](/tools/guard/command-reference/) | Enforce architectural rules and boundaries. |
| [`mago lint`](/tools/linter/command-reference/) | Lint for style, correctness, and best practices. |

Utility commands:

| Command | Description |
| :--- | :--- |
| [`mago config`](/guide/configuration/) | Print the merged configuration or its JSON Schema. |
| [`mago init`](/guide/initialization/) | Scaffold a starter `mago.toml`. |
| [`mago list-files`](/guide/list-files/) | List the files Mago will process. |
| [`mago generate-completions`](/guide/generate-completions/) | Print shell completion scripts. |
| [`mago self-update`](/guide/upgrading/) | Replace the installed binary with a newer release. |

## Exit codes

| Code | Meaning |
| :--- | :--- |
| `0` | Success. No issues found. |
| `1` | Issues found that need attention. |
| `2` | Tool error: configuration, I/O, parse failure, etc. |
