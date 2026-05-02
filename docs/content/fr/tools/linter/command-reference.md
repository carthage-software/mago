+++
title = "Référence de commande du linter"
description = "Tous les indicateurs que mago lint accepte."
nav_order = 50
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Référence de commande

```sh
Usage: mago lint [OPTIONS] [PATH]...
```

Les indicateurs globaux doivent venir avant `lint`. Voir l'[aperçu de la CLI](/fundamentals/command-line-interface/) pour la liste globale.

## Arguments

| Argument | Description |
| :--- | :--- |
| `[PATH]...` | Fichiers ou répertoires à linter. Lorsqu'ils sont fournis, ils remplacent les `paths` de `mago.toml` pour cette exécution. |

## Options spécifiques au linter

| Indicateur | Description |
| :--- | :--- |
| `--list-rules` | Liste toutes les règles activées et leur description. |
| `--json` | À utiliser avec `--list-rules` pour produire un dump JSON lisible par machine. |
| `--explain <CODE>` | Affiche la documentation détaillée pour une règle, par exemple `--explain no-redundant-nullsafe`. |
| `--only <CODE>`, `-o` | Exécute uniquement les règles listées. Séparées par des virgules. Remplace la configuration. |
| `--pedantic` | Active toutes les règles, en ignorant les conditions de version PHP et en activant les règles désactivées par défaut. |
| `--semantics`, `-s` | Exécute uniquement l'analyse + la vérification sémantique. Ignore les règles de lint. |
| `--staged` | Linte uniquement les fichiers stagés dans git. Échoue en dehors d'un dépôt git. |
| `--stdin-input` | Lit le contenu du fichier depuis stdin et utilise l'unique argument de chemin pour la recherche de baseline et le rapport. Destiné aux intégrations d'éditeur. |
| `--substitute <ORIG=TEMP>` | Remplace un fichier hôte par un autre pour cette invocation. Destiné aux tests de mutation. Répétable. |
| `-h`, `--help` | Affiche l'aide et quitte. |

Les indicateurs partagés pour le rapport, la correction et les baselines sont documentés sur la page [options de rapport et de correction](/fundamentals/shared-reporting-options/).

## Lecture depuis stdin

Lorsqu'un éditeur ou un IDE redirige le contenu d'un buffer non sauvegardé, vous pouvez linter ce contenu tout en utilisant le vrai chemin de fichier pour la recherche de baseline et la localisation des problèmes :

```sh
cat src/Example.php | mago lint --stdin-input src/Example.php
```

Exactement un argument de chemin est requis. Il est utilisé comme le nom de fichier logique (relatif à l'espace de travail) pour la correspondance de baseline et les diagnostics. Le chemin est normalisé, de sorte que `./src/Example.php` est traité de la même manière que `src/Example.php`. En conflit avec `--staged`.

## Substitution de fichiers

`--substitute ORIG=TEMP` remplace un fichier hôte par un autre pour la durée d'une seule exécution, sans rien écrire sur le disque. Conçu pour les frameworks de test de mutation (Infection et autres) qui produisent une copie mutée d'un fichier source et veulent que le linter évalue la mutation par rapport au reste du projet. Si le linter signale un nouveau problème sur le fichier muté, la mutation peut être tuée sans exécuter la suite de tests.

```sh
mago lint --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

Règles :

- `ORIG` et `TEMP` doivent tous deux être des chemins absolus et les deux fichiers doivent exister.
- `ORIG` doit être un fichier hôte sous l'un de vos `paths` configurés. Les fichiers vendus ou exclus ne peuvent pas être substitués.
- L'indicateur peut être répété pour substituer plusieurs fichiers à la fois.
- En conflit avec `--stdin-input` et `--staged`.

En interne, `TEMP` est ajouté aux chemins hôtes et `ORIG` est ajouté aux exclusions pour cette exécution, de sorte que les règles inter-fichiers continuent de voir la mutation. Les problèmes signalés et les entrées de baseline référencent `TEMP` plutôt que `ORIG`. Les outils de test de mutation comparent généralement le nombre de problèmes entre une exécution propre et l'exécution substituée, ce qui ne change donc pas le flux de travail.
