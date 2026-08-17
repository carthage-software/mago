+++
title = "Configuring extension hosts"
description = "Configure extension commands, worker pools, environments, and safety limits."
nav_order = 30
nav_section = "Extensions"
+++
# Configuring extension hosts

Extension hosts are configured under `[extension-hosts]`. Each named entry describes one command replicated into a pool of identical worker processes.

```toml
[extension-hosts.framework]
enabled = true
command = ["php", ".mago/framework-worker.php"]
workers = 0
working-directory = "."
inherit-environment = true
environment = { APP_ENV = "analysis" }
maximum-payload-size = 67108864
request-timeout-ms = 30000
shutdown-timeout-ms = 250
stderr-tail-size = 65536
```

## Options

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | boolean | `true` | Whether Mago starts this host. A disabled host may omit `command`. |
| `command` | string list | `[]` | Executable followed by literal arguments. No shell parsing is performed. Required when enabled. |
| `workers` | non-negative integer | `0` | `0` selects an adaptive pool with Mago's thread count as its ceiling. A positive value starts exactly that many processes. |
| `working-directory` | path | config directory | Working directory inherited by every worker process. |
| `environment` | string map | `{}` | Variables added to, or replacing values in, the worker environment. |
| `inherit-environment` | boolean | `true` | Whether workers inherit Mago's process environment before `environment` is applied. |
| `maximum-payload-size` | positive integer | `67108864` | Maximum payload bytes accepted in one protocol frame. The maximum representable value is `4294967295`. |
| `request-timeout-ms` | positive integer | `30000` | Deadline for an outer request, including nested metadata and type-comparison requests. |
| `shutdown-timeout-ms` | non-negative integer | `250` | Grace period after shutdown before Mago forcibly terminates a worker. |
| `stderr-tail-size` | non-negative integer | `65536` | Trailing worker standard-error bytes retained for failure diagnostics. |

## Command resolution

`command` is passed directly to the operating system. Shell syntax, redirects, variable expansion, and pipelines are not interpreted.

```toml
# `php` is resolved through PATH. The script resolves from the config directory.
command = ["php", ".mago/worker.php", "--project=storefront"]
```

A bare executable name such as `php`, `node`, or `my-extension-host` is resolved through `PATH`. A relative executable containing a directory component, such as `./tools/extension-host`, resolves from the effective configuration file's directory. Remaining arguments are passed literally; the selected program interprets relative arguments from `working-directory`.

## Adaptive and fixed pools

The default `workers = 0` uses an adaptive pool. It starts at most three processes to avoid unnecessary PHP startup and memory costs, then may grow toward the global Mago thread count when sustained request contention and observed callback time justify more processes. An analyzer host with parallel after-file or targeted work may proactively reach half of that capacity, rounded up, before dispatch.

Use a fixed pool when an extension has a known external constraint:

```toml
[extension-hosts.database-aware]
command = ["php", ".mago/database-worker.php"]
workers = 2
```

A fixed pool does not grow or shrink. Avoid configuring more workers than Mago threads unless the extension spends substantial time awaiting cooperative I/O.

## Environment isolation

To start from an empty environment and provide only explicit values:

```toml
[extension-hosts.isolated]
command = ["/usr/bin/php", ".mago/isolated-worker.php"]
inherit-environment = false
environment = {
  PATH = "/usr/bin:/bin",
  APP_ENV = "analysis",
}
```

Environment isolation is not a security sandbox. The worker retains the permissions of the Mago process.

## Multiple hosts and extensions

Each host has its own command, limits, environment, and worker pool. One worker command may register several logical extensions:

```php
(new Worker(
    LaravelExtension::create(),
    PHPUnitExtension::create(),
))->run();
```

Across all enabled hosts, extension identifiers, linter issue codes, and analyzer plugin selectors must remain unique. Extension identifiers and analyzer plugin selectors are compared ASCII case-insensitively; linter rule codes are case-sensitive. External linter codes and analyzer selectors must not collide with their native counterparts. Every process in one pool must advertise identical registration metadata.

## Analyzer plugin selection

Analyzer plugins declare whether they are enabled by default. Explicitly enable plugins by identifier or alias under `[analyzer]`:

```toml
[analyzer]
plugins = ["acme/laravel"]
```

When `disable-default-plugins = true`, only explicitly listed plugins run. This setting applies to Mago's built-in analyzer plugins and external analyzer plugins together.

## Layered configuration

Extension hosts participate in normal configuration merging. Host entries merge by their table name, so a project can inherit a host and disable or adjust it:

```toml
extends = "../mago.base.toml"

[extension-hosts.framework]
enabled = false
```

See [Configuration](/guide/configuration/) for path resolution and merge precedence.
