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

### `XDG_CONFIG_HOME`

Mago follows the XDG Base Directory Specification. You can use this environment variable to change the directory where Mago looks for its global configuration file. If unset, it defaults to `$HOME/.config`.

- **Example**: `XDG_CONFIG_HOME=/path/to/config mago lint`

## Overriding Configuration

The following environment variables can be used to override settings from the `mago.toml` file.

### `MAGO_PHP_VERSION`

Overrides the `php_version` setting. This is useful for testing your code against different PHP versions without modifying the configuration file.

- **Example**: `MAGO_PHP_VERSION=8.2 mago lint`

### `MAGO_THREADS`

Overrides the `threads` setting, allowing you to control the number of parallel threads Mago uses for tasks like linting and formatting.

- **Example**: `MAGO_THREADS=4 mago lint`

### `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION`

Overrides the `allow_unsupported_php_version` setting. Set to `true` to allow Mago to run on unsupported PHP versions. This is not recommended and may lead to unexpected behavior.

- **Example**: `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION=true mago lint`
