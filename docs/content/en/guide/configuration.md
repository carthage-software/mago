+++
title = "Configuration"
description = "How Mago discovers configuration files, what every option in mago.toml does, and how to share configuration across projects."
nav_order = 40
nav_section = "Guide"
+++
# Configuration

Mago reads its configuration from a single file, typically `mago.toml` in your project root. Run `mago init` to scaffold one, or write it by hand.

This page covers configuration discovery, the `extends` directive, the global options, and the `[source]` and `[parser]` sections. Tool-specific options are documented under each tool's reference page.

## Discovery

When you do not pass `--config`, Mago looks for a config file in this order:

1. The workspace directory (the current working directory, or the path given by `--workspace`).
2. `$XDG_CONFIG_HOME` if set, for example `$XDG_CONFIG_HOME/mago.toml`.
3. `$HOME/.config`, for example `~/.config/mago.toml`.
4. `$HOME`, for example `~/mago.toml`.

In each location it looks for `mago.{toml,yaml,yml,json}` first, then `mago.dist.{toml,yaml,yml,json}`. Format precedence within a single directory is `toml > yaml > yml > json`. The first file found wins, which lets you keep a global config in `~/.config/mago.toml` for projects that have no local one.

## Sharing configuration with `extends`

> Available since Mago 1.25. Earlier versions silently ignore the directive.

The `extends` directive lets one config layer on top of others without copy-pasting. Useful when several projects share a base standard.

```toml
# Single parent
extends = "vendor/some-org/mago-config/mago.toml"

# Or a list, applied left-to-right; each later layer overrides earlier ones
extends = [
  "vendor/some-org/mago-config",     # directory: mago.{toml,yaml,yml,json} inside
  "configs/strict.json",              # mixing formats is fine
  "../shared/team-defaults.toml",
]

# This file's own keys override anything from the layers above
php-version = "8.3"
```

### Resolution

Absolute paths are used as-is. Relative paths resolve against the directory of the file declaring `extends`, not against the current working directory. So `mago --config some/dir/config.toml` with `extends = "base.toml"` looks for `some/dir/base.toml`.

File entries must exist and use a recognised extension (`.toml`, `.yaml`, `.yml`, `.json`). Directory entries are scanned for `mago.{toml,yaml,yml,json}` in that precedence; a directory with no recognised file is skipped with a warning rather than failing the build.

### Effective precedence

Layers are merged later-wins, deepest-first:

1. Built-in defaults.
2. Each `extends` layer, recursively. A parent's own `extends` resolves before its keys apply.
3. The owning file's keys.
4. `MAGO_*` environment variables for the [supported scalars](/guide/environment-variables/).
5. CLI flags such as `--php-version`, `--threads`.

### Merge semantics

Per top-level key:

- Tables and objects are deep-merged. A child can override a single key inside a nested table without redefining the whole table.
- Arrays such as `source.excludes` and per-rule `exclude` lists are concatenated, parent first. If a base config excludes `vendor/`, you keep that exclude and add your own.
- Scalars (strings, numbers, booleans) are overwritten by the child.

```toml
# base.toml
threads = 4
[source]
excludes = ["vendor", "node_modules"]
```

```toml
# project mago.toml
extends = "base.toml"
threads = 8
[source]
excludes = ["build"]   # appended -> ["vendor", "node_modules", "build"]
```

Cycles are detected via canonical-path tracking and surface a clear error rather than recursing forever. Diamond inheritance (A extends B and C, both extend D) processes D once and is fine. Layers can mix formats freely; each is parsed by its own driver and merged at a generic value level before the final document is validated against the schema.

## Global options

These keys live at the root of `mago.toml`.

```toml
version = "1"
php-version = "8.2"
threads = 8
stack-size = 8388608     # 8 MiB
editor-url = "phpstorm://open?file=%file%&line=%line%&column=%column%"
```

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `version` | string | none | Pins the Mago version this project is tested against. Accepts a major (`"1"`), minor (`"1.25"`), or exact (`"1.25.1"`) pin. See [version pinning](#version-pinning). |
| `php-version` | string | latest stable | The PHP version Mago should target for parsing and analysis. `mago init` autodetects this from `composer.json` when possible. |
| `allow-unsupported-php-version` | boolean | `false` | Allow Mago to run on a PHP version it does not officially support. Not recommended. |
| `no-version-check` | boolean | `false` | Silences the warning emitted when the installed binary drifts from the pinned version. Major-version drift is always fatal. |
| `threads` | integer | logical CPUs | Number of threads for parallel work. |
| `stack-size` | integer | 2 MiB | Per-thread stack size in bytes. Minimum 2 MiB, maximum 8 MiB. |
| `editor-url` | string | none | URL template for clickable file paths in terminal output. See [editor integration](#editor-integration). |

### Version pinning

Pinning the version surfaces drift between the installed binary and the project's expectations early, instead of silently producing different output.

Three pin levels:

- **Major pin** (`version = "1"`): any `1.x.y` satisfies the pin. A bump to `2.x` is a hard error because a new major may ship with incompatible defaults, schema changes, or rule behaviour. This is the default `mago init` writes.
- **Minor pin** (`version = "1.25"`): any `1.25.y` satisfies the pin. Drift to a different minor warns; drift across majors is still fatal.
- **Exact pin** (`version = "1.25.1"`): any drift warns; drift across majors is still fatal.

The warning can be silenced with `--no-version-check`, the `MAGO_NO_VERSION_CHECK` environment variable, or `no-version-check = true` in the config. None of those affect major-version drift, which is the entire point of pinning.

To sync the installed binary to the project's pin:

```sh
mago self-update --to-project-version
```

For exact pins, this resolves directly to that release tag. For major or minor pins, Mago scans recent GitHub releases and installs the highest one that satisfies the pin. So `version = "1"` with 2.0 already shipped still installs the latest 1.x release without dragging you forward.

`version` is currently optional. A future Mago release may start warning when it is missing, to prepare projects for the eventual 2.0 upgrade.

## `[source]`

The `[source]` section controls how Mago discovers and processes files.

### Three categories of paths

Mago distinguishes between your code, third-party code, and code to ignore entirely:

- **`paths`** are your source files. Mago analyses, lints, and formats them.
- **`includes`** are dependencies (typically `vendor`). Mago parses them so it can resolve symbols and types, but never analyses, lints, or rewrites them.
- **`excludes`** are paths or globs Mago ignores entirely. They apply to every tool.

If a file matches both `paths` and `includes`, the more specific pattern wins. Exact file paths are most specific, then deeper directory paths, then shallow ones, then glob patterns. When patterns are equally specific, `includes` wins, which lets you explicitly mark a path as a dependency.

```toml
[source]
paths     = ["src", "tests"]
includes  = ["vendor"]
excludes  = ["cache/**", "build/**", "var/**"]
extensions = ["php"]
```

Glob patterns work in all three lists:

```toml
[source]
paths    = ["src/**/*.php"]
includes = ["vendor/symfony/**/*.php"]   # only Symfony from vendor
excludes = [
  "**/*_generated.php",
  "**/tests/**",
  "src/Legacy/**",
]
```

### Reference

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `paths` | string list | `[]` | Directories or globs for your source code. If empty, the entire workspace is scanned. |
| `includes` | string list | `[]` | Directories or globs for third-party code Mago should parse but not modify. |
| `excludes` | string list | `[]` | Globs or paths excluded from every tool. |
| `extensions` | string list | `["php"]` | File extensions treated as PHP. |

### Glob settings

`[source.glob]` tunes how globs match. Available since 1.19.

```toml
[source.glob]
literal-separator = true     # `*` does not match `/`; use `**` for recursion
case-insensitive  = false
backslash-escape  = true     # `\` escapes special characters
empty-alternates  = false    # `{,a}` matches "" and "a" when true
```

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `case-insensitive` | bool | `false` | Match patterns case-insensitively. |
| `literal-separator` | bool | `false` | When `true`, `*` does not match path separators. Use `**` for recursive matching. |
| `backslash-escape` | bool | `true` (false on Windows) | Whether `\` escapes special characters. |
| `empty-alternates` | bool | `false` | Whether empty alternates are allowed. |

> Projects scaffolded by `mago init` set `literal-separator = true`. It makes `*` behave the way most users expect, matching one directory level the same way `.gitignore` does.

### Tool-specific excludes

Each tool has its own optional `excludes`. They are additive: a file is excluded if it matches the global list or the tool-specific list.

```toml
[source]
paths    = ["src", "tests"]
excludes = ["cache/**"]            # all tools

[analyzer]
excludes = ["tests/**/*.php"]      # only the analyzer

[formatter]
excludes = ["src/**/AutoGenerated/**/*.php"]

[linter]
excludes = ["database/migrations/**"]
```

The linter also supports per-rule path exclusions, useful when you want one rule to skip a path while everything else still applies. Glob patterns there require Mago 1.20 or later. The full reference is on the [linter configuration page](/tools/linter/configuration-reference/#per-rule-excludes).

```toml
[linter.rules]
prefer-static-closure = { exclude = ["tests/"] }
no-global             = { exclude = ["**/*Test.php"] }
```

> Use `mago list-files` to verify which files Mago will process. `mago list-files --command formatter` shows what the formatter will touch, `--command analyzer` shows the analyzer's view, and so on.

## `[parser]`

```toml
[parser]
enable-short-tags = false
```

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enable-short-tags` | boolean | `true` | Whether to recognise the short open tag `<?` in addition to `<?php` and `<?=`. Equivalent to PHP's `short_open_tag` ini directive. |

Disable short open tags when your `.php` files contain literal `<?xml` declarations or template fragments that are not actually PHP. With `enable-short-tags = false`, sequences like `<?xml version="1.0"?>` are treated as inline text rather than parse errors. The trade-off: any code that relies on `<?` as a PHP open tag will no longer be recognised.

## Editor integration

Mago can render file paths in diagnostic output as [OSC 8 hyperlinks](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda). Click the path in your terminal and your editor opens the file at the right line and column. Supported terminals include iTerm2, WezTerm, Kitty, Windows Terminal, Ghostty, and a handful of others.

Mago auto-detects the running editor when possible. On macOS it reads `__CFBundleIdentifier`; elsewhere it checks `TERM_PROGRAM`. The following are recognised out of the box:

- PhpStorm, IntelliJ IDEA, WebStorm
- VS Code, VS Code Insiders
- Zed
- Sublime Text

If auto-detection misses, configure the URL explicitly. Precedence runs first-match-wins:

1. `MAGO_EDITOR_URL` environment variable.
2. `editor-url` in `mago.toml`.
3. Auto-detection.

```sh
export MAGO_EDITOR_URL="vscode://file/%file%:%line%:%column%"
```

```toml
editor-url = "phpstorm://open?file=%file%&line=%line%&column=%column%"
```

| Placeholder | Meaning |
| :--- | :--- |
| `%file%` | Absolute path to the file. |
| `%line%` | Line number, 1-based. |
| `%column%` | Column number, 1-based. |

Common templates:

| Editor | Template |
| :--- | :--- |
| VS Code | `vscode://file/%file%:%line%:%column%` |
| VS Code Insiders | `vscode-insiders://file/%file%:%line%:%column%` |
| Cursor | `cursor://file/%file%:%line%:%column%` |
| Windsurf | `windsurf://file/%file%:%line%:%column%` |
| PhpStorm / IntelliJ | `phpstorm://open?file=%file%&line=%line%&column=%column%` |
| Zed | `zed://file/%file%:%line%:%column%` |
| Sublime Text | `subl://open?url=file://%file%&line=%line%&column=%column%` |
| Emacs | `emacs://open?url=file://%file%&line=%line%&column=%column%` |
| Atom | `atom://core/open/file?filename=%file%&line=%line%&column=%column%` |

> Hyperlinks render only when output is a terminal with colours enabled. They are automatically suppressed when output is piped or `--colors=never` is set, so they do not interfere with scripts or CI.

The hyperlinks appear in the `rich` (default), `medium`, `short`, and `emacs` reporting formats. Machine-readable formats (`json`, `github`, `gitlab`, `checkstyle`, `sarif`) are unaffected.

## Tool-specific configuration

Each tool has its own reference page covering its options:

- [Linter](/tools/linter/configuration-reference/)
- [Formatter](/tools/formatter/configuration-reference/)
- [Analyzer](/tools/analyzer/configuration-reference/)
- [Guard](/tools/guard/configuration-reference/)

## Inspecting the merged configuration

`mago config` prints the configuration Mago is actually using, after merging defaults, every `extends` layer, environment variables, and CLI flags. Useful when something is not behaving as expected.

```sh
mago config                       # full config as pretty-printed JSON
mago config --show linter         # only the [linter] section
mago config --show formatter
mago config --default             # the built-in defaults
mago config --schema              # JSON Schema for the whole config
mago config --schema --show linter
```

| Flag | Description |
| :--- | :--- |
| `--show <SECTION>` | Print only one section. Values: `source`, `parser`, `linter`, `formatter`, `analyzer`, `guard`. |
| `--default` | Print built-in defaults instead of the merged result. |
| `--schema` | Print JSON Schema, useful for IDE integration or external tooling. |
| `-h`, `--help` | Print help and exit. |

Global flags must come before `config`. See the [CLI overview](/fundamentals/command-line-interface/) for the full list.
