+++
title = "Zed recipe"
description = "Use Mago as the formatter for PHP files in the Zed editor."
nav_order = 20
nav_section = "Recipes"
+++
# Zed recipe

Wire Mago into [Zed](https://zed.dev) so PHP files are formatted on save.

## Prerequisites

- Mago is installed. See the [installation guide](/guide/installation/) if you have not done that yet.
- The `mago` executable is on your `PATH`. The recommended installers handle this; verify with `which mago`.

## Configuration

Open Zed's `settings.json` (`Cmd + ,` on macOS, `Ctrl + ,` on Linux and Windows, then "Open JSON Settings"). Add the PHP block to the `languages` section, merging with whatever you already have:

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

Passing `--stdin-filepath {buffer_path}` lets Mago apply `[source].excludes` and `[formatter].excludes` from your `mago.toml` to the current buffer, and it produces clearer error messages.

## Usage

Save a `.php` file and Zed will format it via Mago. You can also trigger formatting manually from the Command Palette (`Cmd + Shift + P` or `Ctrl + Shift + P`) by running "Format Buffer".
