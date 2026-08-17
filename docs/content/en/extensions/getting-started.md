+++
title = "Getting started"
description = "Build, configure, and validate a minimal extension with the PHP SDK."
nav_order = 20
nav_section = "Extensions"
+++
# Getting started

This guide uses Mago's bundled PHP SDK to create an extension containing one linter rule that reports `eval` expressions. Extensions may use another language by implementing the same worker protocol.

For a ready-to-customize PHP project, start with the [Mago extension template](https://github.com/carthage-software/mago-extension-template). It includes an extension factory, linter and analyzer examples, corpus tests, PHPUnit tests, Mago configuration, and CI. Clone it, replace the `Acme` placeholders, and use the walkthrough below to understand each component.

## Requirements

- A Composer project using `carthage-software/mago`.
- A PHP version allowed by the installed Mago package. Mago 1.47 supports PHP 8.1 through PHP 8.6.
- A Mago configuration file in the project root.

The Mago Composer package contains both the executable installer and the version-matched PHP SDK, so extension packages should use it as their SDK dependency.

## 1. Create the rule

Create `src/Mago/Linter/Rules/NoEvalRule.php`:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Linter\Rules;

use Mago\Sdk\Linter\LintContext;
use Mago\Sdk\Linter\Rule;
use Mago\Sdk\Linter\RuleDefinition;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Syntax\NodeKind;

final class NoEvalRule implements Rule
{
    public function getDefinition(): RuleDefinition
    {
        return new RuleDefinition(
            code: 'acme/no-eval',
            name: 'No eval',
            description: 'Disallows evaluating dynamically generated PHP code.',
            defaultLevel: Level::Error,
            defaultEnabled: true,
            targets: [NodeKind::EvalConstruct],
        );
    }

    public function lint(LintContext $context): void
    {
        $context->cancellation->throwIfCancelled();
        $context->report(Issue::new(
            'Avoid evaluating dynamically generated PHP code.',
            $context->node->span,
        ));
    }
}
```

`targets` is important: Mago finds matching nodes in Rust and sends only the required syntax subtrees to PHP. The rule is not called for unrelated nodes.

## 2. Create the extension factory

Create `src/Mago/AcmeExtension.php`:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago;

use Acme\Mago\Linter\Rules\NoEvalRule;
use Mago\Sdk\Extension;

final class AcmeExtension
{
    private function __construct() {}

    public static function create(): Extension
    {
        return new Extension(
            identifier: 'acme/project-rules',
            name: 'Acme project rules',
            version: '1.0.0',
            linterRules: [new NoEvalRule()],
        );
    }
}
```

The extension package should own its identifier, name, version, default rules, analyzer plugins, and reducer wiring. Consumers call `create()` rather than reconstructing that registration themselves, so they cannot accidentally omit a rule or advertise inconsistent metadata.

Factory parameters should expose intentional, strictly typed package options. Do not make consumers pass the extension identifier or raw rule and plugin lists.

## 3. Create the worker entrypoint

Create `.mago/acme-worker.php`:

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

The worker entrypoint must reserve standard output for the SDK protocol. Write diagnostics to standard error instead.

## 4. Configure the host

Add an extension host to `mago.toml`:

```toml
[extension-hosts.acme]
command = ["php", ".mago/acme-worker.php"]
```

The table name, `acme`, is a local host-pool name. It is separate from the globally unique identifier advertised by the extension.

With no `workers` value, the pool is adaptive: it starts small and may grow up to Mago's configured thread count. The default working directory is the configuration file's directory, so `.mago/acme-worker.php` resolves from there.

## 5. Validate registration

Start the host and validate its registration:

```sh
mago extension validate
```

Inspect the registered extension and rules:

```sh
mago extension list
mago extension list --json
```

Run the rule:

```sh
mago lint --only acme/no-eval
```

Because the example rule is enabled by default, a normal `mago lint` run also executes it.

## 6. Add analyzer behavior

After creating `src/Mago/Analyzer/AcmePlugin.php` and `src/Mago/Analyzer/Providers/ContainerReturnTypes.php`, import both classes in the extension factory and register them there. The consuming worker entrypoint does not change:

```php
public static function create(): Extension
{
    return new Extension(
        identifier: 'acme/project-rules',
        name: 'Acme project rules',
        version: '1.0.0',
        linterRules: [new NoEvalRule()],
        analyzerPlugins: [new AcmePlugin(new ContainerReturnTypes())],
    );
}
```

The plugin implements `Mago\Sdk\Analyzer\Plugin` and registers its hooks and providers through `PluginRegistry`. Continue with [Analyzer plugins](/extensions/analyzer/overview/).
