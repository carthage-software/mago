+++
title = "Linter"
description = "Ce que fait le linter, en quoi il diffère de l'analyseur et où poursuivre la lecture."
nav_order = 10
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Linter

Le linter de Mago est un catalogue organisé de règles qui détectent les problèmes stylistiques, les incohérences et les odeurs de code. La plupart des problèmes sont livrés avec une correction automatique, vous pouvez donc nettoyer de grandes bases de code avec une seule commande.

## Linter contre analyseur

Les deux outils trouvent des problèmes, mais ils opèrent à des niveaux différents.

Le **linter** examine la *forme* de votre code. Il applique les conventions d'équipe, signale les constructions redondantes et suggère une syntaxe plus moderne. Il n'a pas besoin de savoir ce que fait votre code à l'exécution, seulement à quoi il ressemble dans le source.

L'**analyseur** construit un modèle sémantique de l'ensemble de votre base de code. Il sait quels types les fonctions retournent, quelles propriétés les classes possèdent et ce qui peut lever une exception. Il trouve des impossibilités logiques comme l'appel d'une méthode qui n'existe pas sur le type concerné.

Si votre code était une dissertation, le linter serait la passe de grammaire et l'analyseur la vérification des faits.

## Le vérificateur sémantique

Mago traite les fichiers en trois étapes : analyse, vérification sémantique, lint.

Le parseur est volontairement tolérant. Il peut lire une syntaxe que le compilateur PHP standard rejetterait, y compris des fonctionnalités d'une future version de PHP. Le vérificateur sémantique est la deuxième étape qui détecte les erreurs que le parseur tolérant laisse passer mais que PHP traiterait comme fatales :

- Les types de support d'enum invalides comme `enum Foo: array {}`.
- Les fonctionnalités qui ne sont pas disponibles dans la version PHP configurée, par exemple les hooks de propriété avant PHP 8.4.

Exécutez uniquement le parseur et le vérificateur sémantique avec `--semantics` :

```sh
mago lint -s
```

C'est un remplacement plus rapide et plus complet pour `php -l` et un moyen sans friction d'introduire Mago dans une base de code avant d'activer le catalogue complet de règles.

## Ce que propose le linter

- **Vitesse.** Un cœur Rust et un pipeline basé sur des arènes maintiennent la passe de lint bien en dessous d'une seconde sur la plupart des projets.
- **Configuration par règle.** Chaque règle peut être activée, désactivée ou voir sa sévérité ajustée. Certaines règles ont leurs propres options.
- **Corrections automatiques.** De nombreuses règles fournissent une correction sûre ; passez `--fix` et Mago réécrit les fichiers concernés. Les catégories moins sûres sont protégées par des indicateurs explicites.
- **Intégrations de frameworks.** Des ensembles de règles optionnels spécifiques à Symfony, Laravel, PHPUnit, Doctrine, WordPress et une longue traîne d'autres. Activées par projet, listées sur la [page des intégrations](/tools/linter/integrations/).

## Où aller ensuite

- [Utilisation](/tools/linter/usage/) : comment exécuter `mago lint`.
- [Intégrations](/tools/linter/integrations/) : activer les vérifications spécifiques aux frameworks.
- [Règles](/tools/linter/rules/) : la référence complète pour chaque règle.
- [Référence de configuration](/tools/linter/configuration-reference/) : toutes les options que Mago accepte sous `[linter]`.
- [Référence de commande](/tools/linter/command-reference/) : tous les indicateurs que `mago lint` accepte.
