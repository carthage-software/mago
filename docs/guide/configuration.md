---
title: Configuration Reference
---

# Configuration

Mago is configured using a `mago.toml` file. You can generate a default configuration file using the `mago init` command.

This page details the global configuration options and the `[source]` section. For tool-specific options, see the links at the bottom of this page.

## Configuration File Discovery

When no `--config` flag is provided, Mago searches for a configuration file (`mago.toml`, `mago.yaml`, or `mago.json`) in the following locations, in order:

1. **Workspace directory** — the current working directory (or the path given by `--workspace`)
2. **`$XDG_CONFIG_HOME`** — if the environment variable is set (e.g., `$XDG_CONFIG_HOME/mago.toml`)
3. **`$HOME/.config`** — the default XDG config directory (e.g., `~/.config/mago.toml`)
4. **`$HOME`** — the user's home directory (e.g., `~/mago.toml`)

The first file found wins. This allows you to have a global configuration in `~/.config/mago.toml` that applies when no project-level configuration exists.

## Global Options

These options are set at the root of your `mago.toml` file.

```toml
version = "1"
php-version = "8.2"
threads = 8
stack-size = 8388608 # 8 MB
editor-url = "phpstorm://open?file=%file%&line=%line%&column=%column%"
```

| Option                          | Type      | Default        | Description                                                                                           |
| :------------------------------ | :-------- | :------------- | :---------------------------------------------------------------------------------------------------- |
| `version`                       | `string`  | (none)         | Pins the Mago version this project is tested against. Accepts `"1"` (major), `"1.19"` (minor), or `"1.19.3"` (exact). Minor/patch drift warns; a major-version mismatch is a hard error. See [Version pinning](#version-pinning) below. |
| `php-version`                   | `string`  | Latest stable  | The version of PHP to use for parsing and analysis. Defaults to the latest stable PHP version supported by your Mago release. Use `mago init` to auto-detect from `composer.json`. |
| `allow-unsupported-php-version` | `boolean` | `false`        | Allow Mago to run on unsupported PHP versions. Not recommended.                                       |
| `no-version-check`              | `boolean` | `false`        | Silences the warning emitted when the installed Mago binary drifts from the pinned `version`. Does **not** affect major-version drift; that is always fatal. |
| `threads`                       | `integer` | (logical CPUs) | The number of threads to use for parallel tasks.                                                      |
| `stack-size`                    | `integer` | (see below)    | The stack size in bytes for each thread. Defaults to 2MB, with a minimum of 2MB and a maximum of 8MB. |
| `editor-url`                    | `string`  | (none)         | Editor URL template for clickable file paths in terminal output. See [Editor Integration](#editor-integration) below. |

### Version pinning

:::info Available in Mago 1.20.0+
The `version` field, the `--no-version-check` flag, the `MAGO_NO_VERSION_CHECK` environment variable, `no-version-check = true`, and `mago self-update --to-project-version` are all new in **Mago 1.20.0**. On 1.19.x and earlier, `version` in `mago.toml` is ignored and these flags do not exist.
:::

Setting `version` in `mago.toml` pins the project to a specific Mago release line so that drift between the installed binary and the project's expectations is surfaced early instead of silently producing different output.

Three pin levels are supported:

- **Major pin** (`version = "1"`): any `1.x.y` satisfies the pin. A bump to `2.x` is a **hard error** because a new major may ship with incompatible defaults, schema changes, or rule behaviour that would silently reinterpret your config. This is the default emitted by `mago init`.
- **Minor pin** (`version = "1.19"`): any `1.19.y` satisfies the pin. Drift to another minor (`1.18.x` or `1.20.x`) produces a warning; drift across majors is still a hard error.
- **Exact pin** (`version = "1.19.3"`): any drift (patch or higher) produces a warning; drift across majors is still a hard error.

The warning can be silenced with `--no-version-check`, the `MAGO_NO_VERSION_CHECK` environment variable, or `no-version-check = true` in `mago.toml`. **None of these affect major-version drift**; that is always fatal, which is the entire point of pinning.

To sync the installed binary to the project's pin, run:

```bash
mago self-update --to-project-version
```

For exact pins (`version = "1.19.3"`) this resolves directly to that release tag. For major or minor pins, Mago scans the recent GitHub releases and installs the highest one that satisfies the pin. So if you pinned `version = "1"` and 2.0 has shipped, `--to-project-version` will still install the latest 1.x release without dragging you forward to 2.0. The same holds for a minor pin: `version = "1.14"` with 1.15.x out in the wild installs the latest 1.14.x, not 1.15. The command only fails if no published release at all satisfies the pin.

`version` is currently optional; a future Mago release may start warning when it is missing, to prepare projects for the eventual 2.0 upgrade.

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

### Glob Settings

The `[source.glob]` section controls how glob patterns are matched. Available since Mago 1.19.0.

```toml
[source.glob]
# When true, `*` does not match `/` in paths. Use `**` for recursive matching.
# e.g., `src/*/Test` matches `src/foo/Test` but NOT `src/foo/bar/Test`.
literal-separator = true

# Match patterns case-insensitively.
case-insensitive = false

# Whether `\` escapes special characters in patterns.
backslash-escape = true

# Whether empty alternates are allowed, e.g., `{,a}` matches "" and "a".
empty-alternates = false
```

| Option               | Type   | Default | Description                                                                                   |
| :------------------- | :----- | :------ | :-------------------------------------------------------------------------------------------- |
| `case-insensitive`   | `bool` | `false` | Match patterns case-insensitively.                                                            |
| `literal-separator`  | `bool` | `false` | When `true`, `*` does not match path separators. Use `**` for recursive directory matching.   |
| `backslash-escape`   | `bool` | `false` on Windows, `true` otherwise | Whether `\` escapes special characters in patterns.                                           |
| `empty-alternates`   | `bool` | `false` | Whether empty alternates are allowed (e.g., `{,a}` matches `""` and `"a"`).                   |

:::tip
New projects created with `mago init` automatically set `literal-separator = true`, which is the recommended setting. It makes `*` behave like most users expect (matching a single directory level, like `.gitignore`).
:::

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

## Editor Integration

Mago can make file paths in diagnostic output clickable using [OSC 8 terminal hyperlinks](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda). When configured, clicking a file path in the terminal opens the file directly in your editor at the correct line and column.

This works in terminals that support OSC 8 hyperlinks, including iTerm2, Wezterm, Kitty, Windows Terminal, Ghostty, and others.

### Auto-Detection

Mago automatically detects your editor when running inside a supported terminal. On macOS, this uses the `__CFBundleIdentifier` environment variable set by the running application. On other platforms, the `TERM_PROGRAM` variable is checked.

The following editors are automatically detected:

- PhpStorm / IntelliJ IDEA / WebStorm
- VS Code / VS Code Insiders
- Zed
- Sublime Text

When auto-detection succeeds, clickable file paths work out of the box with no configuration needed.

### Manual Configuration

If auto-detection doesn't work for your setup, or you want to override it, you can set the editor URL explicitly.

The `MAGO_EDITOR_URL` environment variable takes the highest precedence:

```sh
export MAGO_EDITOR_URL="vscode://file/%file%:%line%:%column%"
```

The `editor-url` option in `mago.toml` takes precedence over auto-detection but is overridden by the environment variable:

```toml
editor-url = "phpstorm://open?file=%file%&line=%line%&column=%column%"
```

### Precedence

The editor URL is resolved in the following order (first match wins):

1. `MAGO_EDITOR_URL` environment variable
2. `editor-url` in `mago.toml`
3. Auto-detection from terminal environment

### Supported Placeholders

| Placeholder  | Description                          |
| :----------- | :----------------------------------- |
| `%file%`     | Absolute path to the file            |
| `%line%`     | Line number (1-based)                |
| `%column%`   | Column number (1-based)              |

### Editor URL Templates

| Editor               | Template                                                          |
| :------------------- | :---------------------------------------------------------------- |
| VS Code              | `vscode://file/%file%:%line%:%column%`                            |
| VS Code Insiders     | `vscode-insiders://file/%file%:%line%:%column%`                   |
| Cursor               | `cursor://file/%file%:%line%:%column%`                            |
| Windsurf             | `windsurf://file/%file%:%line%:%column%`                          |
| PhpStorm / IntelliJ  | `phpstorm://open?file=%file%&line=%line%&column=%column%`         |
| Zed                  | `zed://file/%file%:%line%:%column%`                               |
| Sublime Text         | `subl://open?url=file://%file%&line=%line%&column=%column%`       |
| Emacs                | `emacs://open?url=file://%file%&line=%line%&column=%column%`      |
| Atom                 | `atom://core/open/file?filename=%file%&line=%line%&column=%column%` |

:::tip
Hyperlinks are only rendered when output is sent to a terminal with colors enabled. They are automatically disabled when output is piped or when `--colors=never` is used, so they won't interfere with scripts or CI pipelines.
:::

### Supported Formats

Clickable file paths are supported in the following reporting formats:

- `rich` (default), `medium`, `short` — file paths in diagnostic headers
- `emacs` — file paths at the start of each line

Machine-readable formats (`json`, `github`, `gitlab`, `checkstyle`, `sarif`) are not affected.

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
