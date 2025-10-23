---
title: Generating shell completions
---

# Generating shell completions

Mago comes with a subcommand to generate completions for common shells.

## Usage

To get the completions for your shell, run the `generate-completions` command
with the desired shell name as its arguments.

```sh
mago generate-completions fish
```

You can store them in a file to be sourced by your shell or if you want them to
always match your Mago version, simply source them directly. For example:

```sh
mago generate-completions fish | source
```

## Command reference

```sh
Usage: mago generate-completions <SHELL>
```

### Options

| Flag, Alias(es)           | Description             |
| :------------------------ | :---------------------- |
| `-h`, `--help`            | Print help information. |
