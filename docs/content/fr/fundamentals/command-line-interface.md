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
| [`mago ast`](/guide/inspecting-the-ast/) | Affiche l'AST d'un fichier PHP. |
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

## Codes de sortie

| Code | Signification |
| :--- | :--- |
| `0` | Succès. Aucun problème trouvé. |
| `1` | Problèmes trouvés nécessitant attention. |
| `2` | Erreur d'outil : configuration, I/O, échec d'analyse, etc. |
