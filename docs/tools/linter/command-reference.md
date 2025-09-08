---
title: Linter command reference
outline: deep
---

# Command reference

The `mago lint` command is the entry point for all linting-related tasks.

:::tip
For global options that can be used with any command, see the [Command-Line Interface overview](/fundamentals/command-line-interface.md). Remember to specify global options **before** the `lint` command.
:::

```sh
Usage: mago lint [OPTIONS] [PATH]...
```

## Arguments

### `[PATH]...`

Optional. A list of specific files or directories to lint. If you provide paths here, they will be used instead of the `paths` defined in your `mago.toml` configuration.

## Options

| Flag, Alias(es)            | Description                                                                                                                                                                            |
| :------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--list-rules`             | List all enabled linter rules and their descriptions.                                                                                                                                  |
| `--json`                   | Used with `--list-rules` to output the rule information in a machine-readable JSON format.                                                                                             |
| `--explain <RULE_CODE>`    | Provide detailed documentation for a specific linter rule (e.g., `no-redundant-nullsafe`).                                                                                             |
| `--only <RULE_CODE>`, `-o` | Run only a specific, comma-separated list of rules, overriding the configuration file.                                                                                                 |
| `--pedantic`               | Enable all linter rules for the most exhaustive analysis possible. This overrides your configuration, ignores PHP version constraints, and enables rules that are disabled by default. |
| `--semantics`, `-s`        | Perform only the parsing and basic semantic checks without running any lint rules.                                                                                                     |
| `--help`, `-h`             | Print the help summary for the command.                                                                                                                                                |

### Shared Reporting and Fixing Options

The `lint` command shares a common set of options with other Mago tools for reporting, fixing, and baseline management.

[**See the Shared Reporting and Fixing Options documentation.**](/fundamentals/shared-reporting-options.md)
