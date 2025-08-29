---
title: Linter integrations
---

# Integrations

**Mago** includes specialized sets of linting rules designed for popular PHP frameworks and libraries. These integrations allow Mago to provide more intelligent and context-aware feedback for your specific stack.

When an integration is enabled, Mago will automatically activate all the rules associated with it. You can still configure or disable individual rules from an integration in your `[linter.rules]` table if needed.

## Available integrations

Mago is built with the broader PHP ecosystem in mind and includes support for a wide range of tools. Currently, the linter has rules dedicated to:
- Laravel
- PHPUnit
- PSL
- Symfony

## Future integrations

The following integrations are planned for future releases:

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

### Testing frameworks

- Behat
- Codeception
- Pest
- PHPSpec
- PHPUnit

### CMS

- Drupal
- Magento
- WordPress

### ORMs

- Cycle
- Doctrine

## Enabling integrations

You can enable integrations in your `mago.toml` file under the `[linter]` table.

```toml
[linter]
# Enable the Symfony and PHPUnit integrations
integrations = ["symfony", "phpunit"]
```
