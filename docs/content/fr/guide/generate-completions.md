+++
title = "Complétions du shell"
description = "Générer des complétions pour bash, zsh, fish, elvish ou PowerShell."
nav_order = 70
nav_section = "Guide"
+++
# Complétions du shell

`mago generate-completions <shell>` affiche un script de complétion pour le shell demandé. Sauvegardez-le là où votre shell s'y attend, ou redirigez-le directement pour qu'il corresponde toujours à la version installée de Mago.

```sh
mago generate-completions fish
mago generate-completions fish | source              # fish, ad-hoc
mago generate-completions zsh > ~/.zfunc/_mago      # zsh, persisted
mago generate-completions bash > /etc/bash_completion.d/mago
```

Shells pris en charge : `bash`, `zsh`, `fish`, `elvish`, `powershell`.

## Référence

```sh
Usage: mago generate-completions <SHELL>
```

| Argument | Description |
| :--- | :--- |
| `<SHELL>` | L'un de `bash`, `zsh`, `fish`, `elvish`, `powershell`. |

| Drapeau | Description |
| :--- | :--- |
| `-h`, `--help` | Affiche l'aide et quitte. |
