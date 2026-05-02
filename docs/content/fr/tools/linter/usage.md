+++
title = "Utilisation du linter"
description = "Façons courantes d'utiliser mago lint, y compris la correction automatique et l'exécution d'une seule règle."
nav_order = 20
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Utilisation

Le point d'entrée est `mago lint`. Il exécute le linter sur les fichiers source déclarés dans `mago.toml` (ou sur les arguments que vous passez sur la ligne de commande).

## Linter tout le projet

```sh
mago lint
```

Mago analyse le projet en parallèle et signale chaque problème qu'il trouve.

## Appliquer les corrections automatiques

La plupart des règles fournissent une correction sûre. Pour réécrire les fichiers concernés sur place :

```sh
mago lint --fix
```

Pour prévisualiser les corrections sous forme de diff unifié sans toucher au disque :

```sh
mago lint --fix --dry-run
```

Pour exécuter le formateur sur chaque fichier que le correcteur a réécrit, ajoutez `--format-after-fix` :

```sh
mago lint --fix --format-after-fix
```

Les corrections moins sûres sont protégées. Utilisez `--potentially-unsafe` pour activer les corrections qui peuvent nécessiter une revue rapide, et `--unsafe` pour celles qui peuvent modifier le comportement. Combiné à `--dry-run`, vous pouvez voir exactement ce qui changerait avant de valider.

## Exécuter une seule règle (ou quelques-unes)

`--only` exécute uniquement les règles listées et ignore le reste. Plus rapide que d'exécuter le catalogue complet, utile pour une adoption progressive.

```sh
mago lint --only no-empty
mago lint --only no-empty,use-compound-assignment
```

Si vous voulez que toutes les règles s'exécutent mais ne voir que les problèmes pour un sous-ensemble de codes, utilisez plutôt `--retain-code`. Voir les [options de rapport et de correction](/fundamentals/shared-reporting-options/) pour la liste complète des indicateurs de contrôle des rapports.

## Linter des fichiers spécifiques

Passez des chemins après la sous-commande pour limiter l'exécution à ces fichiers ou répertoires. Utile dans les hooks de pre-commit pour les modifications stagées.

```sh
mago lint src/Service/PaymentProcessor.php
mago lint src/Service tests/Unit
```

La liste complète des indicateurs se trouve dans la [référence de commande](/tools/linter/command-reference/).
