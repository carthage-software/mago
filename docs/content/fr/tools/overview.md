+++
title = "Outils"
description = "Chaque outil de Mago, ce qu'il fait et où approfondir."
nav_order = 10
nav_section = "Tools"
+++
# Outils

Mago regroupe plusieurs outils dans un binaire. Chaque commande choisit son parseur et ses règles. Les outils partagent les réglages de fichiers et les rapports.

## [Formateur](/tools/formatter/overview/)

Un formateur de code déterministe. Il produit une sortie stable et conventionnelle qui suit [PER-CS](https://www.php-fig.org/per/coding-style/) par défaut et prend en charge des préréglages pour les styles PSR-12, Laravel et Drupal. Aucune option à régler à l'aveuglette, aucun débat.

## [Linter](/tools/linter/overview/)

Un catalogue organisé de règles couvrant la correction, la cohérence, la clarté, la redondance, la sûreté, la sécurité et quelques autres préoccupations. La plupart des problèmes sont accompagnés d'une correction automatique. Les intégrations de frameworks ajoutent des règles spécifiques à Symfony, Laravel, PHPUnit, Doctrine et d'autres.

## [Analyseur](/tools/analyzer/overview/)

Un moteur d'analyse statique qui détecte les erreurs de types et les bugs logiques avant l'exécution. Compatible avec les annotations Psalm et PHPStan, avec prise en charge des génériques, des types conditionnels et du raffinement de flux.

## [Guard architectural](/tools/guard/overview/)

Applique les règles de dépendance et les conventions structurelles. Utile lorsque vous voulez interdire certains chemins `use`, codifier les frontières entre couches ou affirmer que le code d'une partie du projet n'importe jamais le code d'une autre.

## [Formateur Twig](/tools/formatter/twig/)

`mago twig:fmt` formate Twig avec ses propres réglages `[twig.formatter]`. Le texte du template reste intact ; le HTML n'est pas reformaté. Le formateur PHP et `mago fix` ne le lancent pas.
