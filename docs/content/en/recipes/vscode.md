+++
title = "Visual Studio Code recipe"
description = "Use Mago as the formatter for PHP files in VS Code."
nav_order = 10
nav_section = "Recipes"
+++
# Visual Studio Code recipe

VS Code does not yet have an official Mago extension, so this recipe wires Mago in through the [Custom Local Formatters](https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters) extension.

## Prerequisites

- Mago is installed. See the [installation guide](/guide/installation/) if you have not done that yet.
- The `mago` executable is on your `PATH`. The recommended installers handle this; verify with `which mago`.

## Configuration

### Install the bridge extension

1. Open the Extensions view (`Ctrl+Shift+X`).
2. Search for `Custom Local Formatters`.
3. Install the extension by `jkillian`.

### Configure `settings.json`

1. Open the user `settings.json`. The Command Palette (`Ctrl+Shift+P`) entry "Open User Settings (JSON)" will take you there.
2. Add the following blocks (merge with what you already have):

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

Save the file. Restart VS Code if the formatter does not pick up the change.

## Usage

With `editor.formatOnSave` enabled, PHP files are formatted by Mago on every save. You can also run "Format Document" from the Command Palette manually.

## Alternative: Run On Save

If you would rather invoke Mago directly instead of going through VS Code's formatter API, the [Run On Save](https://marketplace.visualstudio.com/items?itemName=emeraldwalk.RunOnSave) extension is a good fit. This is useful when the project ships its own Mago binary, since the command runs in your workspace and applies your `mago.toml`, including exclude rules.

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

After saving a PHP file, VS Code runs Mago against that file using your workspace binary.
