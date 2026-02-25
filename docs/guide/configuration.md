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
| `php-version`                   | `string`  | Latest stable  | The version of PHP to use for parsing and analysis. Defaults to the latest stable PHP version supported by your Mago release. Use `mago init` to auto-detect from `composer.json`. |
| `allow-unsupported-php-version` | `boolean` | `false`        | Allow Mago to run on unsupported PHP versions. Not recommended.                                       |
| `threads`                       | `integer` | (logical CPUs) | The number of threads to use for parallel tasks.                                                      |
| `stack-size`                    | `integer` | (see below)    | The stack size in bytes for each thread. Defaults to 2MB, with a minimum of 2MB and a maximum of 8MB. |

## `[source]` Section

This section configures how Mago discovers and processes files in your project.

### Understanding `paths`, `includes`, and `excludes`

Mago distinguishes between **your code** (what you want to check and format) and **dependencies** (code you need for context but don't want to modify):

- **`paths`** = Your source code - files that Mago will **actively process**:
  - ✓ Analyzed for type errors and logic issues
  - ✓ Linted for code quality and style violations
  - ✓ Formatted to match your code style

- **`includes`** = Dependencies and vendor code - files that Mago will **parse for context only**:
  - ✓ Parsed to understand symbols, classes, functions, and types
  - ✗ NOT analyzed for issues
  - ✗ NOT linted
  - ✗ NOT formatted
  - Example: `vendor` directory, third-party libraries, framework code

- **`excludes`** = Paths or patterns to **completely skip**:
  - Applies globally to ALL tools (linter, formatter, analyzer, guard)
  - Files matching these patterns won't be processed or parsed at all
  - Example: cache directories, build artifacts, generated files

:::tip
If a file matches both `paths` and `includes`, the more specific pattern takes precedence:
- Exact file paths (e.g., `src/b.php`) are most specific
- Deeper directory paths (e.g., `src/foo/bar/`) are more specific than shallow ones (e.g., `src/`)
- Directory paths are more specific than glob patterns (e.g., `src/*.php`)

If patterns have equal specificity, `includes` takes precedence. This allows you to explicitly override the file type for specific paths when needed.
:::

### Basic Example

```toml
[source]
# Your application code - will be analyzed, linted, and formatted
paths = ["src", "tests"]

# Vendor dependencies - only parsed for type information
includes = ["vendor"]

# Completely ignored by all tools
excludes = ["cache/**", "build/**", "var/**"]

# File extensions to treat as PHP
extensions = ["php"]
```

### Glob Pattern Support

Both `paths`, `includes`, and `excludes` support glob patterns:

```toml
[source]
# Use glob patterns to target specific files
paths = ["src/**/*.php"]
includes = ["vendor/symfony/**/*.php"]  # Only Symfony from vendor
excludes = [
    "**/*_generated.php",      # Any generated file
    "**/tests/**",             # All test directories
    "src/Legacy/**",           # Specific legacy code
]
```

### Configuration Reference

| Option       | Type       | Default   | Description                                                                                                                                                                    |
| :----------- | :--------- | :-------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `paths`      | `string[]` | `[]`      | Directories or glob patterns for **your source code**. These files will be analyzed, linted, and formatted. If empty, the entire workspace is scanned.                        |
| `includes`   | `string[]` | `[]`      | Directories or glob patterns for **dependencies** (e.g., `vendor`). These files are parsed for symbols and types but are NOT analyzed, linted, or formatted.                  |
| `excludes`   | `string[]` | `[]`      | Glob patterns or paths to **completely exclude** from all tools. These files won't be processed or parsed at all.                                                             |
| `extensions` | `string[]` | `["php"]` | File extensions to treat as PHP files.                                                                                                                                         |

### Tool-Specific Excludes

In addition to the global `excludes` option, each tool (linter, formatter, analyzer, guard) has its own `excludes` option for tool-specific exclusions.

**Tool-specific excludes are additive** - files are excluded if they match EITHER the global `source.excludes` OR the tool-specific excludes.

```toml
[source]
paths = ["src", "tests"]
excludes = ["cache/**"]  # Excluded from ALL tools

[analyzer]
# Additionally exclude test files from analysis only
# (they'll still be linted and formatted)
excludes = ["tests/**/*.php", "src/**/tests/**"]

[formatter]
# Additionally exclude auto-generated code from formatting only
# (it will still be analyzed and linted)
excludes = ["src/**/AutoGenerated/**/*.php"]

[linter]
# Additionally exclude database migrations from linting only
excludes = ["database/migrations/**"]
```

The linter also supports per-rule path exclusions, allowing you to skip individual rules for specific files or directories while still applying other rules. See the [Linter Configuration Reference](/tools/linter/configuration-reference.md#per-rule-path-exclusions) for details.

```toml
[linter.rules]
# Don't enforce static closures in test files
prefer-static-closure = { exclude = ["tests/"] }
```

:::tip Using `mago list-files`
Use the `mago list-files` command to see which files will be processed:
```sh
# See all files in your project
mago list-files

# See which files the formatter will process
mago list-files --command formatter

# See which files the analyzer will process
mago list-files --command analyzer
```
This helps verify your `paths`, `includes`, and `excludes` configuration is working as expected.
:::

## `[parser]` Section

This section configures how Mago parses PHP code, including lexer-level settings that affect tokenization behavior.

### Example

```toml
[parser]
enable-short-tags = false
```

### Configuration Reference

| Option              | Type      | Default | Description                                                                                                                                                                                             |
| :------------------ | :-------- | :------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `enable-short-tags` | `boolean` | `true`  | Whether to enable PHP short open tags (`<?`). When disabled, only `<?php` and `<?=` are recognized as PHP open tags. Equivalent to PHP's `short_open_tag` ini directive.                               |

### When to Disable Short Open Tags

You might want to disable short open tags if:

- Your project contains XML files with `.php` extensions that use `<?xml` declarations
- You're working with template files that mix PHP and XML/HTML containing `<?` sequences
- Your coding standards require explicit `<?php` tags for clarity
- You want to match the behavior of PHP installations where `short_open_tag` is disabled

:::warning
When `enable-short-tags` is `false`, sequences like `<?xml version="1.0"?>` will be treated as inline text rather than causing parse errors. However, any code using `<?` as a PHP open tag will no longer be recognized as PHP code.
:::

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

You can also output the JSON schema for the configuration using the `--schema` flag. This is useful for generating documentation, IDE integration, or validation tooling.

```sh
# Output the JSON schema for the entire configuration
mago config --schema

# Output the JSON schema for a specific section
mago config --schema --show linter
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
| `--show <SECTION>` | Display only a specific section of the configuration. <br/>**Values:** `source`, `parser`, `linter`, `formatter`, `analyzer` |
| `--default`        | Show the default configuration values instead of the current merged configuration.                                 |
| `--schema`         | Output JSON schema instead of configuration values. Useful for documentation and IDE integration.                  |
| `-h`, `--help`     | Print help information.                                                                                            |
