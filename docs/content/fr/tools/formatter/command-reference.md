+++
title = "Référence de commande du formateur"
description = "Tous les indicateurs que mago format accepte."
nav_order = 40
nav_section = "Tools"
nav_subsection = "Formatter"
+++
# Référence de commande

```sh
Usage: mago format [OPTIONS] [PATH]...
```

`mago fmt` est un alias pour `mago format`. Les deux fonctionnent.

Les indicateurs globaux doivent venir avant `format`. Voir l'[aperçu de la CLI](/fundamentals/command-line-interface/) pour la liste globale.

## Arguments

| Argument | Description |
| :--- | :--- |
| `[PATH]...` | Fichiers ou répertoires à formater. Lorsqu'ils sont fournis, ils remplacent les `paths` de `mago.toml` pour cette exécution. |

```sh
mago fmt src/index.php tests/
```

## Options

| Indicateur | Description |
| :--- | :--- |
| `--dry-run`, `-d` | Imprime un diff unifié des changements qui seraient effectués, sans rien écrire. |
| `--check`, `-c` | Vérifie que chaque fichier source est déjà formaté. Sort `0` en cas de correspondance, `1` si un fichier changerait. |
| `--stdin-input`, `-i` | Lit le source depuis stdin, le formate, imprime le résultat sur stdout. |
| `--stdin-filepath <PATH>` | Chemin logique du buffer stdin. Nécessite `--stdin-input`. Vérifié par rapport à `source.excludes` et `formatter.excludes` ; en cas de correspondance, l'entrée est renvoyée telle quelle. Remplace également `<stdin>` dans les messages de diagnostic. |
| `--staged`, `-s` | Formate uniquement les fichiers stagés dans git et les re-stage. Conçu pour les hooks de pre-commit. |
| `-h`, `--help` | Affiche l'aide et quitte. |
