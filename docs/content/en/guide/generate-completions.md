+++
title = "Shell completions"
description = "Generate completions for bash, zsh, fish, elvish, or PowerShell."
nav_order = 70
nav_section = "Guide"
+++
# Shell completions

`mago generate-completions <shell>` prints a completion script for the shell you ask for. Save it where your shell expects, or pipe it directly so it always matches the installed Mago version.

```sh
mago generate-completions fish
mago generate-completions fish | source              # fish, ad-hoc
mago generate-completions zsh > ~/.zfunc/_mago      # zsh, persisted
mago generate-completions bash > /etc/bash_completion.d/mago
```

Supported shells: `bash`, `zsh`, `fish`, `elvish`, `powershell`.

## Reference

```sh
Usage: mago generate-completions <SHELL>
```

| Argument | Description |
| :--- | :--- |
| `<SHELL>` | One of `bash`, `zsh`, `fish`, `elvish`, `powershell`. |

| Flag | Description |
| :--- | :--- |
| `-h`, `--help` | Print help and exit. |
