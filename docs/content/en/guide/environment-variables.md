+++
title = "Environment variables"
description = "Every environment variable Mago reads, what it does, and where it sits in the precedence chain."
nav_order = 50
nav_section = "Guide"
+++
# Environment variables

Mago reads a small set of environment variables. Some override `mago.toml` keys, the rest control the runtime (logging, colours, config-file lookup).

## Runtime

### `MAGO_LOG`

Logging level. Useful when debugging an unexpected result.

Values: `trace`, `debug`, `info`, `warn`, `error`.

```sh
MAGO_LOG=trace mago lint
```

### `NO_COLOR`

Set to anything truthy to disable all coloured output. Follows the [no-color.org](https://no-color.org/) convention.

```sh
NO_COLOR=1 mago lint
```

### `FORCE_COLOR`

Set to anything truthy to force coloured output even when stdout is not a terminal. Takes precedence over `NO_COLOR`. Follows the [force-color.org](https://force-color.org/) convention.

```sh
FORCE_COLOR=1 mago lint | less -R
```

### `XDG_CONFIG_HOME`

Mago follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/) for finding a global config when no project-level file exists. The fallback chain is:

1. `$XDG_CONFIG_HOME/mago.toml` (if set).
2. `$HOME/.config/mago.toml`.
3. `$HOME/mago.toml`.

Setting `XDG_CONFIG_HOME` changes the first lookup directory.

```sh
XDG_CONFIG_HOME=/path/to/config mago lint
```

## The reserved `MAGO_` prefix

Mago reserves the `MAGO_` prefix for itself. Only the variables documented on this page are officially recognised. Anything else prefixed `MAGO_` is reserved for internal use and may be silently ignored or repurposed in a future release.

> Earlier versions auto-mapped every `MAGO_*` variable into the configuration tree, so something like `MAGO_LINT=1` would crash with an "unknown field" error. Mago 1.25 narrowed this to the explicit list below.

## Configuration overrides

These variables override the matching key in `mago.toml`. They cover top-level scalars only; there is no env-var support for nested settings like individual rule levels. Use the config file (or an `extends` layer) for those.

### `MAGO_PHP_VERSION`

Overrides `php-version`. Useful for testing the same code against multiple PHP versions without editing the config.

```sh
MAGO_PHP_VERSION=8.2 mago lint
```

### `MAGO_THREADS`

Overrides `threads`.

```sh
MAGO_THREADS=4 mago lint
```

### `MAGO_STACK_SIZE`

Overrides `stack-size`, in bytes. Out-of-range values are clamped to the supported window (2 MiB minimum, 8 MiB maximum).

```sh
MAGO_STACK_SIZE=8388608 mago lint
```

### `MAGO_EDITOR_URL`

Overrides `editor-url` and the auto-detected editor URL. Highest-precedence input for clickable file paths in diagnostic output. See the [editor integration section](/guide/configuration/#editor-integration) for supported templates.

```sh
MAGO_EDITOR_URL="phpstorm://open?file=%file%&line=%line%&column=%column%" mago lint
```

### `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION`

Overrides `allow-unsupported-php-version`. Set to `true` to let Mago run on a PHP version it does not officially support. Not recommended.

```sh
MAGO_ALLOW_UNSUPPORTED_PHP_VERSION=true mago lint
```

### `MAGO_NO_VERSION_CHECK`

Overrides `no-version-check`. Set to `true` to silence the warning emitted when the installed binary drifts from the version pinned in `mago.toml`. Major-version drift remains fatal regardless of this variable: the whole point of a major pin is to stop runs across incompatible config schemas.

```sh
MAGO_NO_VERSION_CHECK=true mago lint
```
