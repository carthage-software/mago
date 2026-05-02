+++
title = "Initialization"
description = "Generate a starter mago.toml interactively from your project's existing layout."
nav_order = 30
nav_section = "Guide"
+++
# Initialization

Run `mago init` in your project root and answer a few questions. The command writes a `mago.toml` tuned to the project it found.

```sh
mago init
```

If a `composer.json` is present, Mago offers to read it and pre-fill the source paths, PHP version, and any framework integrations the linter should enable. Accept the suggestion when nothing unusual is going on. Otherwise the command falls back to a manual walkthrough.

## What it asks

When there is no `composer.json` or you choose to configure things by hand, the prompts cover:

- **Source paths.** The directories Mago analyses, lints, and formats. These end up in the `paths` array.
- **Dependency paths.** Third-party code Mago should read for context but never modify, typically `vendor`. Stored as `includes`.
- **Excludes.** Directories or glob patterns to skip entirely (build artifacts, generated files, caches). Stored as `excludes`.
- **PHP version.** The version your code targets, used for syntax checks and rule applicability.
- **Linter integrations.** Framework-specific rules to enable. Pick from the list on the [integrations page](/tools/linter/integrations/).
- **Formatter preset.** Choose a preset (Default, PSR-12, Laravel, Drupal) or customise individual formatter options on the spot.

When the prompts finish, the command writes `mago.toml` to the working directory. The [configuration reference](/guide/configuration/) documents every option the file supports.

## Reference

```sh
Usage: mago init
```

| Flag | Description |
| :--- | :--- |
| `-h`, `--help` | Print help and exit. |

For global options that apply to every Mago command, see the [CLI overview](/fundamentals/command-line-interface/). Global flags must come before the subcommand name.
