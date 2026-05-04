+++
title = "Baseline"
description = "Capturer les problèmes existants pour que Mago ne signale que les nouveaux, avec deux variantes pour différents compromis précision/résilience."
nav_order = 20
nav_section = "Fondamentaux"
+++
# Baseline

Un fichier baseline enregistre les problèmes qui existent dans votre base de code à l'instant présent et indique à Mago de les ignorer lors des exécutions futures. Les nouveaux problèmes introduits après la baseline restent signalés. Utile lors de l'adoption de Mago dans un projet qui compte déjà des centaines ou milliers de problèmes, ou lorsque vous étalez un durcissement de règles sur plusieurs PR.

## Un fichier par outil

Le linter et l'analyseur portent chacun leur propre baseline car les problèmes qu'ils signalent sont différents. Noms conventionnels :

- Linter : `lint-baseline.toml`
- Analyseur : `analysis-baseline.toml`

La commande `mago ast` signale des erreurs d'analyse syntaxique et ne prend pas en charge de baseline.

## Générer une baseline

```sh
mago lint --generate-baseline --baseline lint-baseline.toml
mago analyze --generate-baseline --baseline analysis-baseline.toml
```

La commande exécute l'outil, collecte tous les problèmes trouvés et les sérialise dans le fichier TOML spécifié.

## Utiliser une baseline

```sh
mago lint --baseline lint-baseline.toml
mago analyze --baseline analysis-baseline.toml
```

Quand une baseline est utilisée, Mago :

1. Trouve chaque problème dans le code courant.
2. Les compare à la baseline.
3. Supprime les correspondances.
4. Ne signale que ce qui reste.

Vous pouvez aussi définir le chemin de la baseline dans `mago.toml` pour ne pas avoir à passer `--baseline` à chaque fois :

```toml
[linter]
baseline = "lint-baseline.toml"

[analyzer]
baseline = "analysis-baseline.toml"
```

## Deux variantes

Mago prend en charge deux formes de baseline avec des compromis différents entre précision et résilience.

### Loose (par défaut)

Regroupe les problèmes par `(fichier, code, message)` et stocke un comptage. Résiliente aux décalages de numéros de ligne : si vous reformatez ou insérez du code au-dessus d'un problème, la baseline correspond toujours tant que le même type de problème se produit encore.

```toml
variant = "loose"

[[issues]]
file    = "src/Service/PaymentProcessor.php"
code    = "possibly-null-argument"
message = "Argument #1 of `process` expects `Order`, but `?Order` was given."
count   = 2
```

### Strict

Stocke les plages de lignes exactes par problème. Précise, mais la baseline devient obsolète à chaque décalage des numéros de ligne, donc vous régénérez souvent.

```toml
variant = "strict"

[[entries."src/Service/PaymentProcessor.php".issues]]
code = "possibly-null-argument"
start_line = 42
end_line   = 42

[[entries."src/Service/PaymentProcessor.php".issues]]
code = "possibly-null-argument"
start_line = 87
end_line   = 90
```

### Quand choisir laquelle

| Variante | Idéale pour | Compromis |
| :--- | :--- | :--- |
| Loose | La plupart des projets, pipelines CI | Résiliente au refactoring, moins précise. |
| Strict | Quand le suivi exact des lignes importe | Précise, mais nécessite une régénération fréquente. |

Définissez la variante pour les nouveaux fichiers de baseline dans `mago.toml` :

```toml
[linter]
baseline = "lint-baseline.toml"
baseline-variant = "loose"   # ou "strict"

[analyzer]
baseline = "analysis-baseline.toml"
baseline-variant = "loose"
```

Le réglage n'affecte que la génération. À la lecture d'une baseline existante, Mago détecte la variante depuis l'en-tête `variant` du fichier.

### Compatibilité ascendante

Les fichiers baseline écrits par des versions plus anciennes de Mago (avant la prise en charge des variantes) n'ont pas d'en-tête `variant`. Mago les traite comme strictes et affiche un avertissement recommandant de régénérer le fichier pour qu'il acquière l'en-tête.

## Sauter la baseline temporairement

```sh
mago lint --ignore-baseline
mago analyze --ignore-baseline
```

Utile quand vous voulez voir les problèmes actuellement supprimés par la baseline, par exemple pour en nettoyer une partie.

## Garder la baseline propre

Quand vous corrigez un problème faisant partie de la baseline, son entrée devient obsolète. Mago détecte cela et signale les entrées obsolètes. Régénérez pour nettoyer :

```sh
mago lint --generate-baseline --baseline lint-baseline.toml
```

Passez `--backup-baseline` pour conserver le fichier précédent sous `lint-baseline.toml.bkp` avant écrasement.

## JSON Schema

Si vous construisez de l'outillage ou une intégration IDE qui doit analyser ou générer des fichiers de baseline, récupérez le schéma :

```sh
mago config --schema --show baseline
```

La sortie est un JSON Schema (draft 2020-12) couvrant les deux variantes.
