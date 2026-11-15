+++
title = "Drupal recipe"
description = "Check a Drupal project with the official drupal/mago extension."
nav_order = 70
nav_section = "Recipes"
+++
# Drupal recipe

For Drupal, use the official [`drupal/mago`](https://www.drupal.org/project/mago) extension. It is the recommended way to run Mago on a Drupal project, and it brings what Mago alone doesn't know about Drupal:

- linter rules for the parts of Drupal's coding standards that Mago's own rules don't cover, many of them with fixes;
- analyzer plugins that know Drupal's services, entities, config and plugins, and report Drupal-specific mistakes;
- a configuration preset that sets up the formatter, the linter and the analyzer for Drupal.

Together with Mago's own tools, it replaces phpcs, Coder, PHPStan and phpstan-drupal.

The extension lives on drupal.org and has its own [documentation](https://project.pages.drupalcode.org/mago/). Its install guide is the reference; the steps below are the short version.

## Install

```sh
composer require --dev carthage-software/mago "drupal/mago:^1.0@alpha"
```

`drupal/mago` has no stable release yet, so Composer needs the `@alpha` flag for now.

## Configure

Create `mago.toml` next to `composer.json` and extend the extension's preset:

```toml
extends = "vendor/drupal/mago/drupal.mago.toml"
php-version = "8.3"

[source]
paths = ["web/modules/custom", "web/themes/custom"]
includes = ["vendor", "web/core", "web/modules/contrib"]
```

In a module, theme or profile repository, set `paths = ["."]`. In a Drupal core checkout, extend `vendor/drupal/mago/drupal-core.mago.toml` instead.

## Run

Use the Mago binary that Composer installed, so that it matches the extension's protocol version:

```sh
vendor/bin/mago lint
vendor/bin/mago analyze
```

## Built-in Drupal settings

Mago also has a `drupal` [linter integration](/tools/linter/integrations/) and a `drupal` [formatter preset](/tools/formatter/configuration-reference/). You don't need to set either: the extension's preset picks the formatter preset for you, and it replaces the two rule exceptions the linter integration makes with its own rules.

Report Drupal-specific problems and requests in the [extension's issue queue](https://www.drupal.org/project/issues/mago).
