+++
title = "Référence de commande de l'analyseur"
description = "Tous les indicateurs que mago analyze accepte."
nav_order = 20
nav_section = "Tools"
nav_subsection = "Analyzer"
+++
# Référence de commande

```sh
Usage: mago analyze [OPTIONS] [PATHS]...
```

`mago analyse` est un alias pour `mago analyze`. Les deux fonctionnent.

Les indicateurs globaux doivent venir avant `analyze`. Voir l'[aperçu de la CLI](/fundamentals/command-line-interface/) pour la liste globale.

## Arguments

| Argument | Description |
| :--- | :--- |
| `[PATHS]...` | Fichiers ou répertoires à analyser. Lorsqu'ils sont fournis, ils remplacent les `paths` de `mago.toml` pour cette exécution. |

## Options spécifiques à l'analyseur

| Indicateur | Description |
| :--- | :--- |
| `--no-stubs` | Ignore les stubs intégrés de la bibliothèque standard PHP. À utiliser uniquement lorsque vous avez une raison. |
| `--staged` | Analyse uniquement les fichiers stagés dans git. Échoue en dehors d'un dépôt git. |
| `--stdin-input` | Lit le contenu du fichier depuis stdin et utilise l'unique argument de chemin pour la recherche de baseline et le rapport. Destiné aux intégrations d'éditeur. |
| `--substitute <ORIG=TEMP>` | Remplace un fichier hôte par un autre pour cette invocation. Destiné aux tests de mutation. Répétable. |
| `--watch` | Exécute en continu, ré-analysant aux changements de fichiers. Voir [mode watch](#watch-mode). |
| `--list-codes` | Liste tous les codes de problèmes de l'analyseur en JSON. |
| `-h`, `--help` | Affiche l'aide et quitte. |

Les indicateurs partagés pour le rapport, la correction et les baselines sont documentés sur la page [options de rapport et de correction](/fundamentals/shared-reporting-options/).

## Lecture depuis stdin

Pour les intégrations d'éditeur et d'IDE qui redirigent le contenu d'un buffer non sauvegardé :

```sh
cat src/Example.php | mago analyze --stdin-input src/Example.php
```

Exactement un argument de chemin est requis. Il est utilisé comme nom de fichier logique (relatif à l'espace de travail) pour la correspondance de baseline et les diagnostics. Le chemin est normalisé, donc `./src/Example.php` est traité de la même façon que `src/Example.php`. En conflit avec `--staged` et `--watch`.

## Substitution de fichiers

`--substitute ORIG=TEMP` remplace un fichier hôte par un autre pour la durée d'une seule exécution sans rien écrire sur le disque. Conçu pour les frameworks de tests de mutation (Infection et autres) qui produisent une copie mutée d'un fichier source et veulent que l'analyseur évalue la mutation par rapport au reste du projet. Si l'analyseur signale une nouvelle erreur sur le fichier muté, la mutation peut être tuée sans exécuter la suite de tests.

```sh
mago analyze --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

Règles :

- `ORIG` et `TEMP` doivent tous deux être des chemins absolus et les deux fichiers doivent exister.
- `ORIG` doit être un fichier hôte sous l'un de vos `paths` configurés. Les fichiers vendus ou exclus ne peuvent pas être substitués.
- L'indicateur peut être répété pour substituer plusieurs fichiers à la fois.
- En conflit avec `--stdin-input` et `--staged`.

En interne, `TEMP` est ajouté aux chemins hôtes et `ORIG` est ajouté aux exclusions pour cette exécution, de sorte que l'inférence de types inter-fichiers continue de voir la mutation. Les problèmes signalés et les entrées de baseline référencent `TEMP` plutôt que `ORIG`.

## Mode watch

`--watch` maintient l'analyseur en cours d'exécution et le ré-exécute chaque fois qu'un fichier PHP de l'espace de travail est créé, modifié ou supprimé.

```sh
mago analyze --watch
```

### Redémarrage automatique

L'analyseur surveille également les fichiers qui modifient sa propre configuration :

- Le `mago.toml` chargé (ou la configuration que Mago a récupérée).
- Le fichier de baseline référencé depuis `[analyzer].baseline`.
- `composer.json` et `composer.lock`.

Lorsque l'un de ces fichiers change, l'analyseur redémarre avec la configuration rechargée. Vous pouvez donc modifier `mago.toml`, sauvegarder, et la passe d'analyse suivante utilise les nouveaux paramètres sans redémarrage manuel.

Si aucun fichier de configuration n'existe lorsque le mode watch démarre, l'analyseur surveille la création de tout fichier de configuration pris en charge (`mago.toml`, `mago.yaml`, `mago.json`, …) et redémarre lorsqu'un fichier apparaît.

Appuyez sur **Ctrl+C** pour arrêter la surveillance.
