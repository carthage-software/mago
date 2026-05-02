+++
title = "Formateur"
description = "Ce que fait le formateur et comment il produit une sortie déterministe."
nav_order = 10
nav_section = "Tools"
nav_subsection = "Formatter"
+++
# Formateur

Un formateur déterministe pour PHP. Exécutez-le sur un fichier et vous obtenez la même sortie quel que soit le style actuel du fichier. Arrêtez de débattre des espaces, commencez à lire le code.

## Comment ça fonctionne

Mago emprunte l'approche analyse-et-réimpression utilisée par Prettier, `rustfmt` et Black :

1. Analyser le source en un AST.
2. Jeter le formatage original (espaces, sauts de ligne, indentation, tout).
3. Réimprimer l'AST de zéro selon un ensemble fixe de règles, [PER-CS](https://www.php-fig.org/per/coding-style/) par défaut.

La sortie est identique pour un AST donné quel que soit le style d'entrée. Le comportement à l'exécution du code est préservé exactement : l'AST fait un aller-retour, seule la représentation de surface change.

## Ce que vous obtenez

- **Un style cohérent** sur l'ensemble du projet. Le formateur est volontairement opinionné.
- **PER-CS par défaut**, avec des préréglages optionnels pour les styles PSR-12, Laravel et Drupal.
- **Sûr.** Le formateur est limité aux changements qui ne peuvent pas modifier le comportement du programme.
- **Rapide.** Un cœur Rust et un pipeline basé sur des arènes maintiennent la passe de formatage bien en dessous d'une seconde sur la plupart des projets.

## Où aller ensuite

- [Utilisation](/tools/formatter/usage/) : comment exécuter `mago format`.
- [Format ignore](/tools/formatter/format-ignore/) : pragmas pour ignorer le formatage des fichiers, des régions ou des instructions individuelles.
- [Référence de configuration](/tools/formatter/configuration-reference/) : toutes les options que vous pouvez définir.
- [Référence de commande](/tools/formatter/command-reference/) : tous les indicateurs que `mago format` accepte.
