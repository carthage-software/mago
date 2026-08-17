+++
title = "Packaging and compatibility"
description = "Ship an extension package with a project-owned worker entrypoint."
nav_order = 30
nav_section = "Extensions"
nav_subsection = "Development"
+++
# Packaging and compatibility

The Mago Composer package contains both the executable installer and its version-matched PHP SDK. An extension package should depend on `carthage-software/mago`; users should not install a separate SDK package.

## Package layout

A typical package contains reusable extension objects, not a worker binary:

```text
acme-mago-extension/
├── composer.json
├── Justfile
├── src/
│   └── Mago/
│       ├── AcmeExtension.php
│       ├── Analyzer/
│       │   ├── AcmePlugin.php
│       │   ├── Hooks/
│       │   └── Providers/
│       ├── Linter/
│       │   └── Rules/
│       └── Worker/
└── tests/
    └── corpus/
```

Keep the extension factory at the package root, then group Mago integrations by subsystem and role. The examples throughout this section follow this layout.

Example `composer.json`:

```json
{
  "name": "acme/mago-extension",
  "type": "library",
  "require": {
    "php": "^8.1",
    "carthage-software/mago": "^1.47"
  },
  "autoload": {
    "psr-4": {
      "Acme\\Mago\\": "src/Mago/"
    }
  }
}
```

Mago 1.47 is the first release that contains `Mago\Sdk`. Use the narrowest Mago constraint compatible with the SDK surface tested by the package.

## Expose a package-owned factory

Applications should construct the complete extension through a package-owned factory:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago;

use Acme\Mago\Analyzer\AcmePlugin;
use Acme\Mago\Analyzer\Providers\ContainerReturnTypes;
use Acme\Mago\Linter\Rules\NoEvalRule;
use Mago\Sdk\Extension;

final class AcmeExtension
{
    private function __construct() {}

    public static function create(): Extension
    {
        $containerProvider = new ContainerReturnTypes();

        return new Extension(
            identifier: 'acme/framework',
            name: 'Acme framework support',
            version: '1.0.0',
            linterRules: [new NoEvalRule()],
            analyzerPlugins: [new AcmePlugin($containerProvider)],
        );
    }
}
```

When a package exposes factory arguments, keep them strictly typed, discoverable by IDEs, and available before registration. They avoid an extra protocol cycle and let projects use their existing configuration sources.

The package, not the consuming project, owns the identifier, version, default rules, analyzer plugins, and shared state. Do not accept raw capability lists in the factory: doing so lets users accidentally disable part of the extension or create registrations that differ between workers.

## Let the project own the worker

The consuming project creates an entrypoint such as `.mago/extensions.php`:

```php
<?php

declare(strict_types=1);

use Acme\Mago\AcmeExtension;
use Mago\Sdk\Worker;

require dirname(__DIR__) . '/vendor/autoload.php';

(new Worker(
    AcmeExtension::create(),
))->run();
```

It then configures the command:

```toml
[extension-hosts.php]
command = ["php", ".mago/extensions.php"]
```

The project-owned entrypoint can combine several packages in one host and express application-specific options. A package-supplied executable would make composition and configuration harder.

## One host, several extensions

`Worker` accepts one or more `Extension` values. Each extension declares its stable identifier, human name, package version, rules, plugins, and optional reducer.

Across one Mago run:

- extension identifiers must be unique;
- linter issue codes must be unique;
- analyzer plugin identifiers and aliases must be unique across external and native plugins;
- registrations must be identical in every process of a pool.

The extension version is package metadata shown during inspection. It is not the binary protocol version.

## Compatibility boundary

The supported authoring API consists of symbols marked `@api` under `Mago\Sdk` and their members not marked `@internal`. The entire `Mago\Sdk\Internal` namespace is excluded. Do not instantiate protocol frames, readers, writers, transports, caches, or registered-callback wrappers.

Mago and the bundled SDK share an internal binary protocol version and are distributed as a matched pair. PHP extension packages exchange public DTOs and interfaces with the SDK; they should not implement the wire format or depend on its numeric message kinds.

Follow semantic-version constraints for the Mago package and run integration tests against the minimum and newest supported versions. An independently implemented non-PHP SDK must track Mago's versioned wire protocol; compatibility of the PHP authoring API alone does not guarantee frame compatibility.

## PHP and dependencies

The SDK supports the PHP versions declared by the installed Mago Composer package and uses Revolt for cooperative worker I/O. Extension packages may use their own dependencies, but should avoid booting an entire application or service container in every worker unless required.

Because the worker is a trusted external process, it runs as the same operating-system user and retains that user's filesystem and network permissions. Host environment configuration is operational hygiene, not a sandbox.

## Release checklist

- Validate Composer metadata and autoloading.
- Format, lint, and analyze the extension's PHP code.
- Run unit and end-to-end Mago fixtures.
- Test Linux, macOS, and Windows worker startup.
- Check registration with `mago extension list --json`.
- Benchmark a representative application with and without the extension.
- Document every factory option and its performance implications.
- Constrain the first supported Mago version accurately.
- Keep standard output protocol-clean.
