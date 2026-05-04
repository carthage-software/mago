+++
title = "Premiers pas"
description = "Ce qu'est Mago, ce qu'il fait, et où aller ensuite."
nav_order = 10
nav_section = "Guide"
+++
# Premiers pas

Mago est une chaîne d'outils PHP écrite en Rust. Un seul binaire couvre ce qui demande habituellement trois ou quatre outils distincts.

Il inclut :

- Un [formateur](/tools/formatter/overview/) qui produit une sortie déterministe et suit PER-CS par défaut.
- Un [linter](/tools/linter/overview/) avec un catalogue soigné de règles réparties en neuf catégories. Beaucoup de corrections s'appliquent automatiquement.
- Un [analyseur statique](/tools/analyzer/overview/) qui détecte les erreurs de type et les bugs de logique avant l'exécution, avec prise en charge des annotations Psalm et PHPStan.
- Un [guard architectural](/tools/guard/overview/) qui applique les règles de dépendance et les conventions structurelles.

Le tout fonctionne avec un binaire unique, sans runtime PHP, sans dépendance Composer, sans installation Java. Un workflow typique ressemble à :

```sh
mago init           # génère un mago.toml de départ
mago lint           # signale les problèmes de style et de justesse
mago format         # réécrit les fichiers selon les règles du formateur
mago analyze        # vérifie les types et détecte les bugs de logique
```

## Où aller ensuite

- [Installation](/guide/installation/) parcourt toutes les méthodes d'installation prises en charge.
- [Initialisation](/guide/initialization/) couvre la configuration interactive `mago init`.
- [Configuration](/guide/configuration/) est la référence de chaque option de `mago.toml`.
- [Le playground](/playground/) exécute l'analyseur Mago complet dans votre navigateur si vous voulez l'essayer sans installer.
