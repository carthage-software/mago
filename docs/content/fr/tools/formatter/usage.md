+++
title = "Utilisation du formateur"
description = "Façons courantes d'utiliser mago format : sur place, dry-run, mode check, stdin et pre-commit."
nav_order = 20
nav_section = "Tools"
nav_subsection = "Formatter"
+++
# Utilisation

`mago format` (alias `mago fmt`) est le point d'entrée. Par défaut, il formate sur place chaque fichier source déclaré dans `mago.toml`.

## Formater le projet

```sh
mago format
```

Les fichiers sont réécrits sur place. Exécutez-le après un pull, avant de valider, ou comme étape dans la CI.

## CI : vérifier sans réécrire

Dans une étape d'intégration continue, vous voulez généralement vérifier que le projet est déjà formaté sans rien modifier. L'indicateur `--check` fait exactement cela :

```sh
mago format --check
```

Sort `0` lorsque chaque fichier est déjà formaté, `1` lorsqu'au moins un fichier changerait. Aucune sortie en cas de succès, donc reste silencieux dans le chemin heureux.

## Prévisualiser les changements

Pour voir ce que le formateur ferait sans rien écrire sur le disque, demandez un dry run :

```sh
mago format --dry-run
```

La sortie est un diff unifié des changements proposés.

## Fichiers ou répertoires spécifiques

Passez des chemins après la sous-commande pour limiter l'exécution :

```sh
mago format src/Service.php
mago format src/ tests/
```

## Lire depuis stdin

Utile pour rediriger un buffer depuis un éditeur ou un autre outil. Lit depuis stdin, imprime le résultat formaté sur stdout.

```sh
cat src/Service.php | mago format --stdin-input
```

Les intégrations d'éditeur devraient aussi passer le chemin du buffer pour que les exclusions s'appliquent et que les messages d'erreur d'analyse nomment le vrai fichier :

```sh
cat src/Service.php | mago format --stdin-input --stdin-filepath src/Service.php
```

Si le chemin correspond à un motif d'exclusion, l'entrée est renvoyée telle quelle. Les chemins relatifs et absolus sont tous deux acceptés.

## Pre-commit (uniquement les fichiers stagés)

`--staged` formate uniquement les fichiers actuellement stagés dans git, puis les re-stage. Conçu pour les hooks de pre-commit où vous voulez éviter de toucher les changements non stagés de l'arbre de travail.

```sh
mago format --staged
```

La [recette de pre-commit](/recipes/pre-commit-hooks/) décrit une configuration complète de hook.

La liste complète des indicateurs se trouve dans la [référence de commande](/tools/formatter/command-reference/).
