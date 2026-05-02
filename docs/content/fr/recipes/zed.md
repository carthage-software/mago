+++
title = "Recette Zed"
description = "Utiliser Mago comme formateur pour les fichiers PHP dans l'éditeur Zed."
nav_order = 20
nav_section = "Recettes"
+++
# Recette Zed

Brancher Mago dans [Zed](https://zed.dev) pour que les fichiers PHP soient formatés à la sauvegarde.

## Prérequis

- Mago est installé. Voir le [guide d'installation](/guide/installation/) si ce n'est pas encore fait.
- L'exécutable `mago` est dans votre `PATH`. Les installateurs recommandés s'en chargent ; vérifiez avec `which mago`.

## Configuration

Ouvrez le `settings.json` de Zed (`Cmd + ,` sous macOS, `Ctrl + ,` sous Linux et Windows, puis « Open JSON Settings »). Ajoutez le bloc PHP à la section `languages`, en fusionnant avec ce que vous avez déjà :

```json
{
    "languages": {
        "PHP": {
            "format_on_save": "on",
            "formatter": {
                "external": {
                    "command": "mago",
                    "arguments": ["format", "--stdin-input", "--stdin-filepath", "{buffer_path}"]
                }
            }
        }
    }
}
```

Passer `--stdin-filepath {buffer_path}` permet à Mago d'appliquer `[source].excludes` et `[formatter].excludes` de votre `mago.toml` au buffer courant, et cela produit des messages d'erreur plus clairs.

## Utilisation

Sauvegardez un fichier `.php` et Zed le formatera via Mago. Vous pouvez aussi déclencher le formatage manuellement depuis la palette de commandes (`Cmd + Shift + P` ou `Ctrl + Shift + P`) en lançant « Format Buffer ».
