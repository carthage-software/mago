+++
title = "Helix recipe"
description = "Wire Mago into the Helix editor as the formatter for PHP."
nav_order = 30
nav_section = "Recipes"
+++
# Helix recipe

Use Mago as the formatter for PHP files in the [Helix editor](https://helix-editor.com/).

## Prerequisites

- Mago is installed. See the [installation guide](/guide/installation/) if you have not done that yet.
- The `mago` executable is on your `PATH`. The recommended installers handle this; you can verify with `which mago`.

## Configuration

Add a few lines to your Helix `languages.toml`:

- On Linux and macOS the file is usually at `~/.config/helix/languages.toml`.
- On Windows it is usually at `%AppData%\helix\languages.toml`.
- Or a per-project configuration file can be created at `.helix/languages.toml`.

Create the file if it does not exist, then append:

```toml
# Set Mago as the formatter (this assumes your configuration file is in your current working directory).
formatter = { command = "mago", args = ["--config", "%sh{pwd}/mago.toml", "format", "--stdin-input"] }
# If you want to use a different configuration file, uncomment this line and replace the path.
# formatter = { command = "mago", args = ["--config", "%sh{pwd}/path/to/mago.toml", "format", "--stdin-input"] }

# Set to true to format automatically on save.
auto-format = true
```

This overrides Helix's default formatter for PHP and enables formatting on save.

## Usage

With `auto-format = true`, Mago runs every time you save (`:write` or `:w`). You can also trigger formatting manually with `:format` (or `:fmt`) in command mode.

To verify the setup, open a `.php` file, misalign some code, and save. The code should snap into place.
