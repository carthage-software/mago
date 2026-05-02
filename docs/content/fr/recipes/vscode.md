+++
title = "Recette Visual Studio Code"
description = "Utiliser Mago comme formateur pour les fichiers PHP dans VS Code."
nav_order = 10
nav_section = "Recettes"
+++
# Recette Visual Studio Code

VS Code n'a pas encore d'extension Mago officielle, donc cette recette branche Mago via l'extension [Custom Local Formatters](https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters).

## Prérequis

- Mago est installé. Voir le [guide d'installation](/guide/installation/) si ce n'est pas encore fait.
- L'exécutable `mago` est dans votre `PATH`. Les installateurs recommandés s'en chargent ; vérifiez avec `which mago`.

## Configuration

### Installer l'extension de pont

1. Ouvrez la vue Extensions (`Ctrl+Shift+X`).
2. Recherchez `Custom Local Formatters`.
3. Installez l'extension de `jkillian`.

### Configurer `settings.json`

1. Ouvrez le `settings.json` utilisateur. L'entrée « Open User Settings (JSON) » de la palette de commandes (`Ctrl+Shift+P`) vous y mène.
2. Ajoutez les blocs suivants (fusionnez avec ce que vous avez déjà) :

```json
{
    "customLocalFormatters.formatters": [
        {
            "command": "mago format --stdin-input",
            "languages": ["php"]
        }
    ],

    "[php]": {
        "editor.defaultFormatter": "jkillian.custom-local-formatters",
        "editor.formatOnSave": true
    }
}
```

Sauvegardez le fichier. Redémarrez VS Code si le formateur ne prend pas en compte le changement.

## Utilisation

Avec `editor.formatOnSave` activé, les fichiers PHP sont formatés par Mago à chaque sauvegarde. Vous pouvez aussi lancer « Format Document » manuellement depuis la palette de commandes.

## Alternative : Run On Save

Si vous préférez invoquer Mago directement plutôt que de passer par l'API de formateur de VS Code, l'extension [Run On Save](https://marketplace.visualstudio.com/items?itemName=emeraldwalk.RunOnSave) convient bien. C'est utile quand le projet embarque son propre binaire Mago, puisque la commande s'exécute dans votre workspace et applique votre `mago.toml`, y compris les règles d'exclusion.

```json
{
    "emeraldwalk.runonsave": {
        "commands": [
            {
                "match": "\\.php$",
                "cmd": "${workspaceFolder}/vendor/bin/mago fmt ${relativeFile}"
            }
        ]
    }
}
```

Après avoir sauvegardé un fichier PHP, VS Code lance Mago sur ce fichier en utilisant le binaire du workspace.
