+++
title = "Interface en ligne de commande"
description = "Options globales, sous-commandes, variables d'environnement et codes de sortie."
nav_order = 10
nav_section = "Fondamentaux"
+++
# Interface en ligne de commande

Chaque invocation de Mago suit le schéma `mago [GLOBAL OPTIONS] <SUBCOMMAND>`. Les options globales doivent venir avant la sous-commande.

```sh
mago --colors=never lint        # correct
mago lint --colors=never        # incorrect, --colors est une option globale
```

## Options globales

Ces options s'appliquent à chaque sous-commande et contrôlent le runtime, la découverte de configuration et la sortie.

| Drapeau | Description |
| :--- | :--- |
| `--workspace <PATH>` | Racine du workspace. Par défaut le répertoire courant. |
| `--config <PATH>` | Chemin du fichier de configuration. Sans cela, Mago cherche dans le workspace, `$XDG_CONFIG_HOME`, `~/.config` et `~`. Voir [découverte](/guide/configuration/#discovery). |
| `--php-version <VERSION>` | Surcharge la version PHP configurée, par exemple `8.2`. |
| `--threads <NUMBER>` | Surcharge le nombre de threads. Par défaut, le nombre de CPU logiques. |
| `--allow-unsupported-php-version` | Permet à Mago de s'exécuter sur une version PHP qu'il ne prend pas officiellement en charge. À utiliser avec précaution. |
| `--no-version-check` | Désactive l'avertissement émis lorsque la version installée diverge de la version épinglée du projet par un changement mineur ou patch. Une divergence majeure reste fatale. Voir [épinglage de version](/guide/configuration/#version-pinning). |
| `--colors <WHEN>` | Quand colorer la sortie : `always`, `never` ou `auto` (par défaut). |
| `-h`, `--help` | Affiche l'aide et quitte. |
| `-V`, `--version` | Affiche la version installée et quitte. |

## Variables d'environnement

La plupart des surcharges de configuration utilisent le préfixe `MAGO_*` et sont documentées sur la [page des variables d'environnement](/guide/environment-variables/). Les deux que vous êtes le plus susceptible de définir au quotidien sont :

| Variable | Rôle |
| :--- | :--- |
| `MAGO_LOG` | Filtre de log pour la sortie de tracing. Valeurs : `trace`, `debug`, `info`, `warn`, `error`. |
| `MAGO_EDITOR_URL` | Modèle d'URL pour les chemins de fichiers cliquables dans la sortie du terminal. Voir [intégration éditeur](/guide/configuration/#editor-integration). |

## Sous-commandes

Les outils principaux :

| Commande | Description |
| :--- | :--- |
| [`mago analyze`](/tools/analyzer/command-reference/) | Analyse statique : erreurs de type, bugs de logique. |
| [`mago cst`](/guide/inspecting-the-cst/) | Affiche l'CST d'un fichier PHP. |
| [`mago fix`](#mago-fix) | Applique les corrections des quatre outils jusqu'à ce qu'aucun changement ne reste possible. |
| [`mago format`](/tools/formatter/command-reference/) | Formate les fichiers PHP. |
| [`mago guard`](/tools/guard/command-reference/) | Applique les règles et frontières architecturales. |
| [`mago lint`](/tools/linter/command-reference/) | Linte pour le style, la justesse et les bonnes pratiques. |

Commandes utilitaires :

| Commande | Description |
| :--- | :--- |
| [`mago config`](/guide/configuration/) | Affiche la configuration fusionnée ou son JSON Schema. |
| [`mago init`](/guide/initialization/) | Génère un `mago.toml` de départ. |
| [`mago list-files`](/guide/list-files/) | Liste les fichiers que Mago va traiter. |
| [`mago generate-completions`](/guide/generate-completions/) | Affiche les scripts de complétion shell. |
| [`mago self-update`](/guide/upgrading/) | Remplace le binaire installé par une release plus récente. |
| `mago version` | Affiche la version de Mago. Identique à `--version`. |

## mago fix

`mago fix [PATHS...]` lance **guard → analyzer → linter → formatter**, puis répète cet ordre jusqu'à ce qu'un passage complet ne modifie aucun fichier. Chaque outil lit les changements de l'outil précédent. Sans chemins explicites, la commande utilise ceux de la configuration.

```sh
mago fix
mago fix src/ tests/ --potentially-unsafe
mago fix --no-analyze --no-guard
```

Par défaut, seules les corrections sûres s'appliquent. La commande respecte la configuration, les exclusions, les suppressions dans le code et la baseline de chaque outil. Les problèmes sans correction autorisée ne bloquent ni les autres outils ni la fin de la commande.

| Option | Description |
| :--- | :--- |
| `--potentially-unsafe` | Autorise les corrections sûres et potentiellement risquées. |
| `--unsafe` | Autorise toutes les corrections. Vérifiez les changements avec soin. |
| `--no-guard` | Ignore le guard. |
| `--no-analyze` | Ignore l'analyzer. |
| `--no-lint` | Ignore le linter. |
| `--no-fmt` | Ignore le formatter. |
| `--ignore-baseline` | Corrige aussi les problèmes masqués par les baselines. |
| `--fail-on-remaining` | Renvoie le code `1` s'il reste des problèmes après les corrections. |
| `--max-passes <NUMBER>` | Limite le nombre de passages complets : `10` par défaut, de `1` à `256`. |

Par défaut, la commande réussit quand il ne reste aucune correction autorisée par le niveau de sûreté choisi, même s'il reste des problèmes. Désactiver les quatre outils ne fait rien et réussit.

Si les corrections ramènent les fichiers à un état déjà rencontré ou atteignent la limite de passages, la commande s'arrête avec le code `1` et signale le problème. Elle conserve les changements déjà faits ; vérifiez les règles et les réglages avant de relancer. Les erreurs d'accès aux fichiers et les autres erreurs des outils arrêtent aussi la commande.

Vous pouvez augmenter `--max-passes` jusqu'à `256`. Si les corrections ne se stabilisent toujours pas après `256` passages, signalez un bug dans Mago.

## Codes de sortie

| Code | Signification |
| :--- | :--- |
| `0` | Succès. `mago fix` n'a plus de correction autorisée à appliquer. |
| `1` | Des problèmes demandent votre attention, ou les corrections ne se stabilisent pas. |
| `2` | Erreur d'outil : configuration, I/O, etc. |
