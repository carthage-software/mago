+++
title = "Intégrations du linter"
description = "Packs de règles spécifiques aux frameworks que vous pouvez activer par projet."
nav_order = 30
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Intégrations

Mago fournit des ensembles de règles spécifiques aux frameworks PHP populaires, aux bibliothèques et aux outils de test. Lorsqu'une intégration est activée, Mago active les règles qui lui sont liées. Les règles individuelles peuvent toujours être ajustées ou désactivées dans `[linter.rules]`.

## Intégrations disponibles

### Frameworks

- CakePHP
- Laminas
- Laravel
- Neutomic
- Spiral
- Symfony
- Tempest
- Yii

### Bibliothèques

- Amphp
- Carbon
- Guzzle
- Monolog
- PSL (PHP Standard Library)
- ReactPHP

### Tests

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

> Certaines de ces intégrations sont des espaces réservés pour de futures règles. La liste des intégrations qui ont actuellement des règles attachées se trouve sur la [page des règles](/tools/linter/rules/#integration-specific-rules).

## Activation des intégrations

Dans `mago.toml`, listez les intégrations que vous voulez sous `[linter].integrations` :

```toml
[linter]
integrations = ["symfony", "phpunit"]
```

La surface de configuration complète est sur la [référence de configuration](/tools/linter/configuration-reference/).
