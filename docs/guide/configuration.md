---
title: Configuration Reference
---

# Configuration

Mago is configured using a `mago.toml` file in the root of your project. You can generate a default configuration file using the `mago init` command.

This page details the global configuration options and the `[source]` section. For tool-specific options, see the links at the bottom of this page.

## Global Options

These options are set at the root of your `mago.toml` file.

```toml
php-version = "8.2"
threads = 8
stack-size = 8388608 # 8 MB
```

| Option                          | Type      | Default        | Description                                                                                           |
| :------------------------------ | :-------- | :------------- | :---------------------------------------------------------------------------------------------------- |
| `php-version`                   | `string`  | `"8.1"`        | The version of PHP to use for parsing and analysis.                                                   |
| `allow-unsupported-php-version` | `boolean` | `false`        | Allow Mago to run on unsupported PHP versions. Not recommended.                                       |
| `threads`                       | `integer` | (logical CPUs) | The number of threads to use for parallel tasks.                                                      |
| `stack-size`                    | `integer` | (see below)    | The stack size in bytes for each thread. Defaults to 2MB, with a minimum of 2MB and a maximum of 8MB. |

## `[source]` Section

This section configures how Mago discovers files to analyze and format.

```toml
[source]
paths = ["src", "tests"]
includes = ["vendor/symfony/http-foundation"]
excludes = ["src/Legacy/**", "**/*_generated.php"]
extensions = ["php", "php8"]
```

| Option       | Type       | Default   | Description                                                                                                                                                                                                                                                                  |
| :----------- | :--------- | :-------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `paths`      | `string[]` | `[]`      | A list of directories to scan for source files. If empty, the entire workspace is scanned.                                                                                                                                                                                   |
| `includes`   | `string[]` | `[]`      | A list of specific files or directories to include for analysis context (e.g., vendor packages). These files will not be formatted, linted, or analyzed directly, but they provide essential context for the analyzer to understand symbols and types from third-party code. |
| `excludes`   | `string[]` | `[]`      | A list of paths or glob patterns to exclude from scanning.                                                                                                                                                                                                                   |
| `extensions` | `string[]` | `["php"]` | A list of file extensions to consider as PHP files.                                                                                                                                                                                                                          |

## Tool-Specific Configuration

For details on configuring the linter, formatter, and analyzer, see their respective reference pages:

- [Linter Configuration](/tools/linter/configuration-reference.md)
- [Formatter Configuration](/tools/formatter/configuration-reference.md)
- [Analyzer Configuration](/tools/analyzer/configuration-reference.md)
- [Guard Configuration](/tools/guard/configuration-reference.md)

## The `config` Command

The `mago config` command is a utility to display the final, merged configuration that Mago is using for the current project.

This is invaluable for debugging your setup, as it shows you the result of combining your `mago.toml` file, any environment variables, and the built-in defaults.

### Usage

Running the command without any options will print the entire configuration object as a pretty-printed JSON object.

```sh
mago config
```

You can inspect a specific part of the configuration using the `--show` flag.

```sh
# Show only the [linter] configuration
mago config --show linter

# Show only the [formatter] configuration
mago config --show formatter
```

### Command reference

:::tip
For global options that can be used with any command, see the [Command-Line Interface overview](/fundamentals/command-line-interface.md). Remember to specify global options **before** the `config` command.
:::

```sh
Usage: mago config [OPTIONS]
```

| Flag, Alias(es)    | Description                                                                                                        |
| :----------------- | :----------------------------------------------------------------------------------------------------------------- |
| `--show <SECTION>` | Display only a specific section of the configuration. <br/>**Values:** `source`, `linter`, `formatter`, `analyzer` |
| `-h`, `--help`     | Print help information.                                                                                            |
