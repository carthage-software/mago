+++
title = "Référence de commande du guard"
description = "Tous les indicateurs que mago guard accepte."
nav_order = 30
nav_section = "Tools"
nav_subsection = "Guard"
+++
# Référence de commande

```sh
Usage: mago guard [OPTIONS] [PATHS]...
```

Les indicateurs globaux doivent venir avant `guard`. Voir l'[aperçu de la CLI](/fundamentals/command-line-interface/) pour la liste globale.

## Arguments

| Argument | Description |
| :--- | :--- |
| `[PATHS]...` | Fichiers ou répertoires à vérifier. Lorsqu'ils sont fournis, ils remplacent les `paths` de `mago.toml` pour cette exécution. |

## Sélection de mode

Ces indicateurs choisissent quelle moitié du guard s'exécute. Ils sont mutuellement exclusifs.

| Indicateur | Description |
| :--- | :--- |
| `--structural` | Exécute uniquement les vérifications structurelles (nommage, modificateurs, héritage). |
| `--perimeter` | Exécute uniquement les vérifications de périmètre (frontières de dépendances, restrictions de couche). |

Si aucun indicateur n'est défini, les deux moitiés s'exécutent, comme `mode = "default"` en configuration. Ces indicateurs remplacent le `mode` configuré. Si l'indicateur correspond au mode configuré, le guard imprime un avertissement de redondance.

## Autres options

| Indicateur | Description |
| :--- | :--- |
| `--no-stubs` | Ignore les stubs intégrés PHP et de bibliothèques. À utiliser uniquement lorsque vous avez une raison. |
| `--stdin-input` | Lit le contenu du fichier depuis stdin et utilise l'unique argument de chemin pour la recherche de baseline et le rapport. Destiné aux intégrations d'éditeur. |
| `--substitute <ORIG=TEMP>` | Remplace un fichier hôte par un autre pour cette invocation. Destiné aux tests de mutation. Répétable. |
| `-h`, `--help` | Affiche l'aide et quitte. |

Les indicateurs partagés pour le rapport, la correction et les baselines sont documentés sur la page [options de rapport et de correction](/fundamentals/shared-reporting-options/). La correction automatique n'est pas actuellement significative pour les problèmes de guard, mais les indicateurs sont acceptés par parité avec les autres outils.

## Lecture depuis stdin

Pour les intégrations d'éditeur qui redirigent le contenu d'un buffer non sauvegardé :

```sh
cat src/Example.php | mago guard --stdin-input src/Example.php
```

Exactement un argument de chemin est requis. Il est utilisé comme nom de fichier relatif à l'espace de travail pour la correspondance de baseline et les diagnostics. Le chemin est normalisé, donc `./src/Example.php` est traité de la même façon que `src/Example.php`. En conflit avec `--substitute`.

## Substitution de fichiers

`--substitute ORIG=TEMP` remplace un fichier hôte par un autre pour la durée d'une seule exécution sans rien écrire sur le disque. Conçu pour les frameworks de tests de mutation qui produisent une copie mutée d'un fichier source et veulent que le guard évalue la mutation par rapport au reste du projet.

```sh
mago guard --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

Règles :

- `ORIG` et `TEMP` doivent tous deux être des chemins absolus et les deux fichiers doivent exister.
- `ORIG` doit être un fichier hôte sous l'un de vos `paths` configurés. Les fichiers vendus ou exclus ne peuvent pas être substitués.
- L'indicateur peut être répété pour substituer plusieurs fichiers à la fois.
- En conflit avec `--stdin-input`.

En interne, `TEMP` est ajouté aux chemins hôtes et `ORIG` est ajouté aux exclusions pour cette exécution, de sorte que l'analyse des dépendances continue de voir la mutation. Les problèmes signalés référencent `TEMP` plutôt que `ORIG`.
