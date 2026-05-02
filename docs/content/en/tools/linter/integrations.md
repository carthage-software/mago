+++
title = "Linter integrations"
description = "Framework-specific rule packs you can enable per project."
nav_order = 30
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Integrations

Mago ships rule sets specific to popular PHP frameworks, libraries, and testing tools. When an integration is enabled, Mago activates the rules tied to it. Individual rules can still be tuned or disabled in `[linter.rules]`.

## Available integrations

### Frameworks

- CakePHP
- Laminas
- Laravel
- Neutomic
- Spiral
- Symfony
- Tempest
- Yii

### Libraries

- Amphp
- Carbon
- Guzzle
- Monolog
- PSL (PHP Standard Library)
- ReactPHP

### Testing

- Behat
- Codeception
- Pest
- PHPSpec
- PHPUnit

### CMS

- Drupal
- Magento
- WordPress

### ORM

- Cycle
- Doctrine

> Some of these are placeholders for future rules. The list of integrations that currently have rules attached is on the [rules page](/tools/linter/rules/#integration-specific-rules).

## Enabling integrations

In `mago.toml`, list the integrations you want under `[linter].integrations`:

```toml
[linter]
integrations = ["symfony", "phpunit"]
```

The full configuration surface is on the [configuration reference](/tools/linter/configuration-reference/).
