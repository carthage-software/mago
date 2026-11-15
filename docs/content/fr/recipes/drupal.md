+++
title = "Recette Drupal"
description = "Vérifier un projet Drupal avec l'extension officielle drupal/mago."
nav_order = 70
nav_section = "Recettes"
+++
# Recette Drupal

Pour Drupal, utilisez l'extension officielle [`drupal/mago`](https://www.drupal.org/project/mago). C'est la façon recommandée d'utiliser Mago sur un projet Drupal, et elle apporte ce que Mago seul ne sait pas de Drupal :

- des règles de linter pour les parties des normes de codage Drupal que les règles de Mago ne couvrent pas, dont beaucoup avec des corrections ;
- des plugins d'analyse qui connaissent les services, entités, configurations et plugins de Drupal, et signalent les erreurs propres à Drupal ;
- un préréglage de configuration qui prépare le formateur, le linter et l'analyseur pour Drupal.

Avec les outils de Mago, elle remplace phpcs, Coder, PHPStan et phpstan-drupal.

L'extension est hébergée sur drupal.org et a sa propre [documentation](https://project.pages.drupalcode.org/mago/). Son guide d'installation fait référence ; les étapes ci-dessous en sont la version courte.

## Installation

```sh
composer require --dev carthage-software/mago "drupal/mago:^1.0@alpha"
```

`drupal/mago` n'a pas encore de version stable, Composer a donc besoin de l'option `@alpha` pour l'instant.

## Configuration

Créez `mago.toml` à côté de `composer.json` et étendez le préréglage de l'extension :

```toml
extends = "vendor/drupal/mago/drupal.mago.toml"
php-version = "8.3"

[source]
paths = ["web/modules/custom", "web/themes/custom"]
includes = ["vendor", "web/core", "web/modules/contrib"]
```

Dans le dépôt d'un module, d'un thème ou d'un profil, indiquez `paths = ["."]`. Dans une copie de Drupal core, étendez plutôt `vendor/drupal/mago/drupal-core.mago.toml`.

## Exécution

Utilisez le binaire Mago installé par Composer, pour qu'il corresponde à la version du protocole de l'extension :

```sh
vendor/bin/mago lint
vendor/bin/mago analyze
```

## Réglages Drupal intégrés

Mago a aussi une [intégration de linter](/tools/linter/integrations/) `drupal` et un [préréglage de formateur](/tools/formatter/configuration-reference/) `drupal`. Vous n'avez besoin d'aucun des deux : le préréglage de l'extension choisit le préréglage de formateur pour vous, et remplace par ses propres règles les deux exceptions que fait l'intégration de linter.

Signalez les problèmes et demandes propres à Drupal dans la [file de tickets de l'extension](https://www.drupal.org/project/issues/mago).
