+++
title = "Recette Helix"
description = "Connecter Mago à l'éditeur Helix comme formateur pour PHP."
nav_order = 30
nav_section = "Recettes"
+++
# Recette Helix

Utiliser Mago comme formateur pour les fichiers PHP dans l'[éditeur Helix](https://helix-editor.com/).

## Prérequis

- Mago est installé. Voir le [guide d'installation](/guide/installation/) si ce n'est pas encore fait.
- L'exécutable `mago` est dans votre `PATH`. Les installateurs recommandés s'en chargent ; vous pouvez vérifier avec `which mago`.

## Configuration

Ajoutez quelques lignes à votre `languages.toml` Helix :

- Sous Linux et macOS, le fichier se trouve généralement à `~/.config/helix/languages.toml`.
- Sous Windows, il se trouve généralement à `%AppData%\helix\languages.toml`.

Créez le fichier s'il n'existe pas, puis ajoutez :

```toml
[[language]]
name = "php"

formatter = { command = "mago", args = ["format", "--stdin-input"] }
auto-format = true
```

Cela surcharge le formateur Helix par défaut pour PHP et active le formatage à la sauvegarde.

## Utilisation

Avec `auto-format = true`, Mago s'exécute à chaque sauvegarde (`:write` ou `:w`). Vous pouvez aussi déclencher le formatage manuellement avec `:format` (ou `:fmt`) en mode commande.

Pour vérifier la configuration, ouvrez un fichier `.php`, désalignez du code, puis sauvegardez. Le code devrait s'aligner.
