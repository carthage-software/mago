---
title: Helix Editor recipe
---

# 🧩 Helix editor recipe

Integrate Mago directly with the [Helix editor](https://helix-editor.com/) for fast, reliable, and automatic code formatting.

## Prerequisites

1.  **Mago Installed**: Ensure you have installed Mago. If not, please follow the [Installation Guide](/guide/installation.md).
2.  **`PATH` Configured**: The `mago` executable must be available in your system's `PATH`. The recommended installation scripts handle this for you. You can verify this by running `which mago` in your terminal.

## Configuration

You only need to add a few lines to your Helix `languages.toml` file.

1.  **Locate your `languages.toml` file**:
    - On Linux & macOS, it's typically at `~/.config/helix/languages.toml`.
    - On Windows, it's typically at `%AppData%\helix\languages.toml`.
      If the file doesn't exist, you can create it.

2.  **Add the Mago formatter configuration**: Add the following TOML block to the end of the file. This will override the default formatter for PHP and enable formatting on save.

    ```toml
    # ~/.config/helix/languages.toml

    [[language]]
    name = "php"

    # Set Mago as the formatter.
    # This assumes your mago.toml file is in your current working directory.
    # If you work on multiple projects, consider using a .helix/languages.toml file in your project directory.
    formatter = { command = "mago", args = ["format", "--stdin-input"] }

    # Set to true to format automatically on save.
    auto-format = true
    ```

## Usage

Once configured, Helix is ready to use Mago.

- If `auto-format = true`, your PHP files will be formatted by Mago every time you save (`:write` or `:w`).
- You can trigger formatting manually at any time by running the `:format` (or `:fmt`) command in Helix's command mode.

To verify the setup, open a `.php` file, misalign some code, and save the file. The code should snap into place as Mago formats it.
