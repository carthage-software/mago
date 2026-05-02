+++
title = "Analyseur"
description = "Ce que fait l'analyseur, en quoi il diffère du linter et où poursuivre la lecture."
nav_order = 10
nav_section = "Tools"
nav_subsection = "Analyzer"
+++
# Analyseur

Un moteur d'analyse statique pour PHP. Il construit un modèle sémantique de l'ensemble du projet, puis parcourt chaque fonction, méthode et expression pour détecter les erreurs de types et les impossibilités logiques avant qu'elles ne s'exécutent.

## Analyseur contre linter

Les deux outils trouvent des problèmes, mais ils opèrent à des niveaux différents.

Le **linter** examine la *forme* du code : problèmes stylistiques, incohérences, odeurs de code. Il n'a pas besoin de savoir ce que le code fait à l'exécution.

L'**analyseur** examine le *sens* du code. Il suit le type de chaque variable à travers chaque branche, sait ce que chaque méthode d'une classe retourne réellement et suit les exceptions qui peuvent se propager. Il détecte les impossibilités comme l'appel d'une méthode qui n'existe pas sur le type concerné, le passage d'un `?Order` là où un `Order` est requis, ou le retour de `null` depuis une fonction annotée comme ne retournant jamais null.

Si votre code était une dissertation, le linter serait la passe de grammaire et l'analyseur la vérification des faits.

## Ce que propose l'analyseur

- **Inférence de types.** L'analyseur comprend le type de chaque expression même lorsque les indications de types sont partielles. Il prend en charge les annotations Psalm et PHPStan, les génériques, les types conditionnels et le raffinement de flux.
- **Conscience de tout le programme.** L'analyse s'exécute à travers le projet, donc les appels à d'autres fichiers révèlent de véritables incompatibilités de signature.
- **Vitesse.** Cœur Rust, parallélisé, exécute un projet entier en quelques secondes.
- **Vérifications heuristiques.** Un ensemble configurable de vérifications consultatives pour les préoccupations de qualité de code qui ne sont pas des erreurs strictes mais qui indiquent souvent des bugs latents.

## Où aller ensuite

- [Référence de commande](/tools/analyzer/command-reference/) : tous les indicateurs que `mago analyze` accepte.
- [Référence de configuration](/tools/analyzer/configuration-reference/) : toutes les options que Mago accepte sous `[analyzer]`.
