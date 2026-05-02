+++
title = "Options de rapport et de correction"
description = "Les drapeaux partagés par lint, analyze et ast pour signaler les problèmes, appliquer les corrections et gérer les baselines."
nav_order = 40
nav_section = "Fondamentaux"
+++
# Options de rapport et de correction

`mago lint`, `mago analyze` et `mago ast` partagent un ensemble de drapeaux pour la manière dont les problèmes sont signalés et les corrections appliquées. Cette page est la référence centrale pour ces drapeaux afin de ne pas les répéter sur chaque page de commande.

## Correction automatique

La plupart des règles du linter et une poignée de vérifications de l'analyseur fournissent des corrections automatiques. Les drapeaux ci-dessous contrôlent comment les corrections sont appliquées et quelles catégories sont éligibles.

| Drapeau | Description |
| :--- | :--- |
| `--fix` | Applique chaque correction sûre aux problèmes trouvés. |
| `--fixable-only`, `-f` | Filtre la sortie aux problèmes pour lesquels une correction automatique est disponible. |
| `--unsafe` | Applique les corrections marquées comme non sûres. Elles peuvent altérer le comportement et nécessitent revue. |
| `--potentially-unsafe` | Applique les corrections marquées comme potentiellement non sûres. Moins risquées que les non sûres mais méritent quand même une revue rapide. |
| `--format-after-fix`, `fmt` | Lance le formateur sur chaque fichier modifié par `--fix`. |
| `--dry-run`, `-d`, `diff` | Prévisualise les corrections sous forme de diff unifié sans rien écrire. |

## Rapport

Comment Mago présente les problèmes trouvés.

| Drapeau | Description |
| :--- | :--- |
| `--sort` | Trie les problèmes signalés par niveau, puis par code, puis par emplacement. |
| `--reporting-target <TARGET>` | Où écrire le rapport. Valeurs : `stdout` (par défaut), `stderr`. |
| `--reporting-format <FORMAT>` | Format de sortie. Voir ci-dessous ; par défaut auto-détecté. |
| `--minimum-fail-level <LEVEL>`, `-m` | Niveau le plus bas qui déclenche un code de sortie non nul. Valeurs : `note`, `help`, `warning`, `error`. Par défaut, la valeur du fichier de configuration, ou `error` si absent. |
| `--minimum-report-level <LEVEL>` | Niveau le plus bas inclus dans le rapport. Les problèmes en dessous sont filtrés avant impression. |
| `--retain-code <CODE>` | Garde uniquement les problèmes ayant le ou les codes donnés. Filtre de rapport, pas filtre d'exécution. Répétable. |

`--retain-code` n'est pas la même chose que `--only` (que seul `mago lint` accepte) :

- `mago lint --only <RULE>` n'exécute que les règles spécifiées. Les autres règles sont entièrement ignorées, ce qui est plus rapide.
- `mago lint --retain-code <CODE>` exécute toutes les règles et filtre la sortie sur les codes listés.

```sh
mago lint --only no-unused-variable                                  # only run that rule
mago lint --retain-code no-unused-variable                           # run everything, show only this code
mago lint --retain-code no-unused-variable --retain-code semantics   # multiple codes
mago analyze --retain-code invalid-argument --retain-code type-mismatch
```

Utilisez `--only` quand vous voulez une boucle de retour rapide sur une règle précise. Utilisez `--retain-code` quand vous voulez une couverture complète mais un rapport ciblé.

### Formats de rapport

Choisissez explicitement avec `--reporting-format` :

- Lisibles par un humain : `rich`, `medium`, `short`, `ariadne`, `emacs`.
- CI / lisibles par machine : `github`, `gitlab`, `json`, `checkstyle`, `sarif`.
- Résumés : `count`, `code-count`.

### Détection automatique

Si `--reporting-format` n'est pas défini, Mago en choisit un selon l'environnement :

| Environnement | Détecté via | Format par défaut |
| :--- | :--- | :--- |
| GitHub Actions | `GITHUB_ACTIONS` | `github` |
| GitLab CI | `GITLAB_CI` | `gitlab` |
| Agents IA de coding | `CLAUDECODE`, `GEMINI_CLI`, `CODEX_SANDBOX`, `OPENCODE_CLIENT` | `medium` |
| Tout le reste | (aucun) | `rich` |

Les pipelines CI obtiennent donc des annotations natives et les agents IA un format économe en jetons sans configuration. Passez `--reporting-format` explicitement pour surcharger.

> La détection automatique est disponible depuis Mago 1.18. Sur 1.17 et antérieur, définissez `--reporting-format=github` ou `--reporting-format=gitlab` explicitement.

## Baseline

Drapeaux pour la gestion des fichiers baseline. Le guide complet se trouve sur la [page baseline](/fundamentals/baseline/).

| Drapeau | Description |
| :--- | :--- |
| `--generate-baseline` | Génère un nouveau fichier baseline capturant chaque problème courant. |
| `--baseline <PATH>` | Utilise la baseline au chemin donné. |
| `--backup-baseline` | À la régénération, copie l'ancienne baseline dans `<file>.bkp` avant écrasement. |
| `--ignore-baseline` | Ignore toute baseline configurée ou spécifiée et signale chaque problème. |
