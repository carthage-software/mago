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
| [`mago cst`](/guide/inspecting-the-cst/) | Print the CST of a PHP file. |
| [`mago fix`](#mago-fix) | Apply fixes from all four tools until no more changes are possible. |
| [`mago format`](/tools/formatter/command-reference/) | Format PHP files. |
| [`mago guard`](/tools/guard/command-reference/) | Enforce architectural rules and boundaries. |
| [`mago lint`](/tools/linter/command-reference/) | Lint for style, correctness, and best practices. |

Utility commands:

| Command | Description |
| :--- | :--- |
| [`mago config`](/guide/configuration/) | Print the merged configuration or its JSON Schema. |
| [`mago init`](/guide/initialization/) | Scaffold a starter `mago.toml`. |
| [`mago inspect-baseline`](/fundamentals/baseline/#inspecting-a-baseline) | Summarise and visualise a baseline file. |
| [`mago list-files`](/guide/list-files/) | List the files Mago will process. |
| [`mago generate-completions`](/guide/generate-completions/) | Print shell completion scripts. |
| [`mago self-update`](/guide/upgrading/) | Replace the installed binary with a newer release. |
| `mago version` | Print Mago's version. Same as `--version`. |

## mago fix

`mago fix [PATHS...]` runs **guard → analyzer → linter → formatter**, then repeats that order until a full pass changes no files. Each tool reads the changes from the previous tool. Without paths, the command uses the configured source paths.

```sh
mago fix
mago fix src/ tests/ --potentially-unsafe
mago fix --no-analyze --no-guard
```

Only safe fixes run by default. The command respects each tool's configuration, excludes, inline suppressions, and baseline. Issues without an allowed fix do not stop the other tools or cause an endless loop.

| Flag | Description |
| :--- | :--- |
| `--potentially-unsafe` | Allow safe and potentially unsafe fixes. |
| `--unsafe` | Allow all fixes. Review the changes carefully. |
| `--no-guard` | Skip the guard. |
| `--no-analyze` | Skip the analyzer. |
| `--no-lint` | Skip the linter. |
| `--no-fmt` | Skip the formatter. |
| `--ignore-baseline` | Apply fixes to issues hidden by each tool's baseline too. |
| `--fail-on-remaining` | Exit with code `1` if issues remain after fixes settle. |
| `--max-passes <NUMBER>` | Limit full passes; defaults to `10` and accepts `1` to `256`. |

By default, the command succeeds when no more fixes are available at the selected safety level, even if issues remain. Disabling all four tools does nothing and succeeds.

If fixes keep returning to an earlier file state, or reach the pass limit, the command stops with code `1` and reports the problem. It keeps the edits already made; review the rules and settings before trying again. File access and other tool errors stop the command too.

You can raise `--max-passes` up to `256`. If fixes still do not settle after `256` passes, report a bug in Mago.

## Exit codes

| Code | Meaning |
| :--- | :--- |
| `0` | Success. `mago fix` found no more allowed fixes. |
| `1` | Issues need attention, or fixes did not settle. |
| `2` | Tool error: configuration, I/O, etc. |
