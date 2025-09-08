---
title: Visual Studio Code recipe
---

# 🧩 Visual Studio Code recipe

Integrate Mago directly into Visual Studio Code for powerful, automatic PHP code formatting.

This guide uses the [Custom Local Formatters](https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters) extension to connect Mago to VS Code's formatting engine.

## Prerequisites

1.  **Mago Installed**: Ensure you have installed Mago by following the [Installation Guide](/guide/installation.md).
2.  **`PATH` Configured**: The `mago` executable must be available in your system's `PATH`. The recommended installation methods configure this for you.

## Configuration

### Install the extension

First, you need to install the bridge extension that allows VS Code to run Mago as a formatter.

1.  Open the **Extensions** view in VS Code (`Ctrl+Shift+X`).
2.  Search for `Custom Local Formatters`.
3.  Install the extension created by **`jkillian`**.

### Configure `settings.json`

Next, you'll configure the extension to use Mago and tell VS Code to use it for PHP files.

1.  Open your user `settings.json` file. You can do this by opening the Command Palette (`Ctrl+Shift+P`) and searching for "Open User Settings (JSON)".
2.  Add the following configuration to your `settings.json`. If you already have these settings, merge them accordingly.

    ```json
    {
      // ... your other settings

      // 1. Define the Mago command for the formatter extension.
      "customLocalFormatters.formatters": [
        {
          "command": "mago format --stdin-input",
          "languages": ["php"]
        }
      ],

      // 2. Configure VS Code to use this extension for PHP files.
      "[php]": {
        // Set the custom formatter as the default for PHP.
        "editor.defaultFormatter": "jkillian.custom-local-formatters",
        // Recommended: automatically format files on save.
        "editor.formatOnSave": true
      }
    }
    ```

3.  Save the `settings.json` file. You may need to restart VS Code for all changes to take effect.

## Usage

Your setup is now complete.

- With `editor.formatOnSave` enabled, your PHP files will be automatically formatted by Mago every time you save.
- You can also manually format a file at any time by opening the command palette (`Ctrl+Shift+P`) and running the **Format Document** command.
