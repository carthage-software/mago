# Integrating Mago with Helix

The Mago formatter can be seamlessly integrated with Helix to provide automatic code formatting within your editor.
This integration is straightforward and only requires a simple configuration in your `languages.toml` file.

Steps:

1. Ensure Mago is installed and available in your system's `PATH`.

2. Configure Helix:
   Add the following configuration to your `languages.toml` file (typically located at `~/.config/helix/languages.toml`):

```toml
[[language]]
name = "php"
formatter = { command = "mago", args = ["fmt", "--stdin-input"] }
# If you want format on save
auto-format = true
```

_That's it!_ Now, whenever you format a PHP file in Helix (typically with `:fmt` or on save), Helix will automatically run the `mago fmt --stdin-input` command to format your code.

> Note: Make sure that the `mago` executable is in your system's `PATH` so that Helix can find it.
