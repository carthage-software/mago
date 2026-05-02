+++
title = "Premiers pas"
description = "Ce qu'est Mago, ce qu'il fait, et où aller ensuite."
nav_order = 10
nav_section = "Guide"
+++
# Premiers pas

Mago est une chaîne d'outils PHP écrite en Rust. Un seul binaire couvre les parties d'un workflow qui prennent habituellement trois ou quatre outils séparés.

Il inclut :

- Un [formateur](/tools/formatter/overview/) qui produit une sortie déterministe et suit PER-CS par défaut.
- Un [linter](/tools/linter/overview/) avec un catalogue soigné de règles réparties en neuf catégories. Beaucoup de corrections s'appliquent automatiquement.
- Un [analyseur statique](/tools/analyzer/overview/) qui attrape les erreurs de type et les bugs de logique avant l'exécution, avec prise en charge des annotations Psalm et PHPStan.
- Un [guard architectural](/tools/guard/overview/) qui applique les règles de dépendance et les conventions structurelles.

Le tout fonctionne avec un binaire unique, sans runtime PHP, sans dépendance Composer, sans installation Java. Un workflow typique ressemble à :

```sh
mago init           # write a starter mago.toml
mago lint           # surface stylistic and correctness issues
mago format         # rewrite files to match the formatter
mago analyze        # type-check and find logic bugs
```

## Où aller ensuite

- [Installation](/guide/installation/) parcourt toutes les méthodes d'installation prises en charge.
- [Initialisation](/guide/initialization/) couvre la configuration interactive `mago init`.
- [Configuration](/guide/configuration/) est la référence de chaque option de `mago.toml`.
- [Le playground](/playground/) exécute l'analyseur Mago complet dans votre navigateur si vous voulez l'essayer sans installer.
