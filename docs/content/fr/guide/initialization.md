+++
title = "Initialisation"
description = "Générer interactivement un mago.toml de départ à partir de l'arborescence existante de votre projet."
nav_order = 30
nav_section = "Guide"
+++
# Initialisation

Lancez `mago init` à la racine de votre projet et répondez à quelques questions. La commande écrit un `mago.toml` adapté au projet trouvé.

```sh
mago init
```

Si un `composer.json` est présent, Mago propose de le lire et de pré-remplir les chemins source, la version PHP et les intégrations de framework que le linter doit activer. Acceptez la suggestion quand rien d'inhabituel ne se passe. Sinon, la commande passe à un parcours manuel.

## Ce que la commande demande

S'il n'y a pas de `composer.json` ou si vous choisissez de configurer manuellement, les questions couvrent :

- **Chemins source.** Les répertoires que Mago analyse, linte et formate. Ils finissent dans le tableau `paths`.
- **Chemins de dépendances.** Le code tiers que Mago doit lire pour le contexte mais ne jamais modifier, généralement `vendor`. Stockés dans `includes`.
- **Exclusions.** Répertoires ou motifs glob à ignorer entièrement (artefacts de build, fichiers générés, caches). Stockés dans `excludes`.
- **Version PHP.** La version que cible votre code, utilisée pour les vérifications de syntaxe et l'applicabilité des règles.
- **Intégrations du linter.** Règles spécifiques à un framework à activer. Choisissez dans la liste de la [page des intégrations](/tools/linter/integrations/).
- **Préréglage du formateur.** Choisissez un préréglage (Default, PSR-12, Laravel, Drupal) ou personnalisez les options du formateur sur le moment.

Quand les questions sont terminées, la commande écrit `mago.toml` dans le répertoire courant. La [référence de configuration](/guide/configuration/) documente toutes les options que le fichier prend en charge.

## Référence

```sh
Usage: mago init
```

| Drapeau | Description |
| :--- | :--- |
| `-h`, `--help` | Affiche l'aide et quitte. |

Pour les options globales qui s'appliquent à toutes les commandes Mago, voir l'[aperçu CLI](/fundamentals/command-line-interface/). Les drapeaux globaux doivent venir avant le nom de la sous-commande.
