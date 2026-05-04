+++
title = "Recette GitHub Actions"
description = "Lancer le formatage, le linting et l'analyse à chaque push et chaque pull request."
nav_order = 50
nav_section = "Recettes"
+++
# Recette GitHub Actions

Un workflow simple qui lance le formateur, le linter et l'analyseur à chaque push et chaque pull request, avec des annotations natives sur les PR.

## Configuration rapide

Créez `.github/workflows/mago.yml` :

```yaml
name: Mago Code Quality

on:
  push:
  pull_request:

jobs:
  mago:
    name: Run Mago Checks
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Set up PHP
        uses: shivammathur/setup-php@v2
        with:
          php-version: "8.4"
          coverage: none
          tools: composer
        env:
          COMPOSER_ALLOW_SUPERUSER: 1

      - name: Install Composer dependencies
        run: composer install --prefer-dist --no-progress

      - name: Set up Mago
        uses: nhedger/setup-mago@v1

      - name: Check formatting
        run: mago format --check

      - name: Lint
        if: success() || failure()
        run: mago lint

      - name: Analyze
        if: success() || failure()
        run: mago analyze
```

Quelques notes sur la structure :

- Séparer `format`, `lint` et `analyze` en étapes distinctes fait remonter les résultats des trois, même quand une étape antérieure échoue. Un seul `run:` combiné ferait court-circuit au premier échec et masquerait le reste.
- `if: success() || failure()` lance l'étape quand le job n'a pas été annulé, ce qui est ce que vous voulez ici. `always()` la lancerait aussi après des échecs de setup.
- Utilisez `mago format --check`, pas `--dry-run`. `--check` quitte avec un code non nul quand des fichiers ont besoin d'être formatés ; `--dry-run` n'affiche qu'un diff et quitte toujours zéro.
- Mago détecte GitHub Actions via la variable d'environnement `GITHUB_ACTIONS` et bascule automatiquement sur `--reporting-format=github`, produisant des annotations natives sur les PR. Aucune configuration supplémentaire requise. Sur 1.17.0 et antérieur, vous devez passer `--reporting-format=github` à `mago lint` et `mago analyze` manuellement.

## Utiliser l'image Docker

Si vous préférez ne pas installer Mago sur le runner, lancez l'[image Docker officielle](/recipes/docker/) en tant que job conteneur :

```yaml
name: Mago Code Quality

on:
  push:
  pull_request:

jobs:
  mago:
    name: Run Mago Checks
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/carthage-software/mago:1
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Check formatting
        run: mago fmt --check

      - name: Lint
        if: success() || failure()
        run: mago lint

      - name: Analyze
        if: success() || failure()
        run: mago analyze
```

L'image n'inclut pas PHP ni Composer. Cela fonctionne très bien pour le formateur et le linter, mais l'analyseur a besoin des dépendances Composer pour résoudre les symboles. Pour l'analyse, préférez l'[approche setup-mago](#quick-setup) avec `composer install` exécuté en premier.
