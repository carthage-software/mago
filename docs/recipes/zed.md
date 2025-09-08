---
title: Zed editor recipe
---

# 🧩 Zed editor recipe

Integrate Mago directly into the [Zed editor](https://zed.dev) for seamless, high-performance code formatting on save.

## Prerequisites

1.  **Mago Installed**: Ensure you have installed Mago. If not, please follow the [Installation Guide](/guide/installation.md).
2.  **`PATH` configured**: The `mago` executable must be in your system's `$PATH`. The recommended installation methods handle this automatically. You can verify this by running `which mago` in your terminal, which should return a path to the executable.

## Configuration

1.  **Open Settings**: Launch Zed and open your `settings.json` file. You can do this by pressing `Cmd + ,` (on macOS) or `Ctrl + ,` (on Linux/Windows) and then clicking "Open JSON Settings".

2.  **Add PHP Configuration**: Add the following JSON block to your `settings.json`. If you already have a `"languages"` section, simply add the `"PHP"` object within it.

    ```json
    {
      // ... other settings you may have
      "languages": {
        "PHP": {
          // Recommended: Format your code automatically whenever you save a file.
          "format_on_save": "on",

          // Configure Mago as the external formatter.
          "formatter": {
            "external": {
              "command": "mago",
              "arguments": ["format", "--stdin-input"]
            }
          }
        }
        // ... other language settings
      }
    }
    ```

## Usage

That's it! With this configuration, Zed will now automatically format your `.php` files with Mago every time you save.

You can also trigger formatting manually at any time by:

- Opening the command palette with `Cmd + Shift + P` or `Ctrl + Shift + P`.
- Typing "Format Buffer" and pressing Enter.
