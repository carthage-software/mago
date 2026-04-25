---
title: Environment Variables
---

# Environment Variables

Mago's behavior can be configured using several environment variables. These variables can be used to override settings defined in the `mago.toml` configuration file.

## General

### `MAGO_LOG`

Sets the logging level for Mago. This is useful for debugging issues or getting more detailed output.

- **Values**: `trace`, `debug`, `info`, `warn`, `error`
- **Example**: `MAGO_LOG=trace mago lint`

### `NO_COLOR`

If this variable is set to any value (e.g., `1`, `true`), it disables all colored output from Mago.

- **Example**: `NO_COLOR=1 mago lint`

See [no-color.org](https://no-color.org/) for more information.

### `FORCE_COLOR`

If this variable is set to any non-empty value (e.g., `1`, `true`), it forces colored output from Mago, even when the output is not a terminal (e.g., when piping to a file or another command).

This takes precedence over `NO_COLOR`.

- **Example**: `FORCE_COLOR=1 mago lint | less -R`

See [force-color.org](https://force-color.org/) for more information.

### `XDG_CONFIG_HOME`

Mago follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/). When no configuration file is found in the workspace, Mago searches for a global configuration file in the following order:

1. `$XDG_CONFIG_HOME/mago.toml` (if `XDG_CONFIG_HOME` is set)
2. `$HOME/.config/mago.toml` (the default XDG config directory)
3. `$HOME/mago.toml` (the user's home directory)

Set this variable to change the first lookup directory:

- **Example**: `XDG_CONFIG_HOME=/path/to/config mago lint`

## Reserved `MAGO_` prefix

The `MAGO_` prefix is reserved for Mago. Only the variables documented on this page are officially recognised; anything else prefixed `MAGO_` is reserved for internal use and may be silently ignored or repurposed in a future release.

:::info Behaviour change in Mago 1.25.0
Earlier versions auto-mapped any `MAGO_*` variable into the configuration tree (so `MAGO_LINT=1` would error out with "unknown field"). Mago 1.25.0 narrows this to the explicit list below; unknown `MAGO_*` variables no longer error and no longer override anything.
:::

## Overriding Configuration

The following top-level scalars can be overridden via environment variables. There is no env-var support for nested settings like individual rule levels. Use `mago.toml` (or an extended config; see [Sharing configuration with `extends`](/guide/configuration#sharing-configuration-with-extends)) for those.

### `MAGO_PHP_VERSION`

Overrides the `php_version` setting. This is useful for testing your code against different PHP versions without modifying the configuration file.

- **Example**: `MAGO_PHP_VERSION=8.2 mago lint`

### `MAGO_THREADS`

Overrides the `threads` setting, allowing you to control the number of parallel threads Mago uses for tasks like linting and formatting.

- **Example**: `MAGO_THREADS=4 mago lint`

### `MAGO_STACK_SIZE`

Overrides the `stack-size` setting (per-thread stack size in bytes). Valid range is enforced by normalisation; out-of-range values are clamped.

- **Example**: `MAGO_STACK_SIZE=8388608 mago lint`

### `MAGO_EDITOR_URL`

Overrides the `editor-url` setting and the auto-detected editor URL. This takes the highest precedence for determining clickable file path URLs in diagnostic output.

See [Editor Integration](/guide/configuration#editor-integration) for supported URL templates.

- **Example**: `MAGO_EDITOR_URL="phpstorm://open?file=%file%&line=%line%&column=%column%" mago lint`

### `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION`

Overrides the `allow-unsupported-php-version` setting. Set to `true` to allow Mago to run on unsupported PHP versions. This is not recommended and may lead to unexpected behavior.

- **Example**: `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION=true mago lint`

### `MAGO_NO_VERSION_CHECK`

_Available in Mago 1.20.0+._ Has no effect on 1.19.x and earlier.

Overrides the `no-version-check` setting. Set to `true` to silence the warning emitted when the installed Mago binary drifts from the `version` pinned in `mago.toml`. Major-version drift is **always fatal** and is not affected by this variable; the whole point of a major pin is to stop runs across incompatible config schemas.

- **Example**: `MAGO_NO_VERSION_CHECK=true mago lint`
