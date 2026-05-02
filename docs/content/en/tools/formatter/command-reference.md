+++
title = "Formatter command reference"
description = "Every flag mago format accepts."
nav_order = 40
nav_section = "Tools"
nav_subsection = "Formatter"
+++
# Command reference

```sh
Usage: mago format [OPTIONS] [PATH]...
```

`mago fmt` is an alias for `mago format`. Both work.

Global flags must come before `format`. See the [CLI overview](/fundamentals/command-line-interface/) for the global list.

## Arguments

| Argument | Description |
| :--- | :--- |
| `[PATH]...` | Files or directories to format. When provided, these replace the `paths` from `mago.toml` for this run. |

```sh
mago fmt src/index.php tests/
```

## Options

| Flag | Description |
| :--- | :--- |
| `--dry-run`, `-d` | Print a unified diff of the changes that would be made, without writing anything. |
| `--check`, `-c` | Verify that every source file is already formatted. Exits `0` on match, `1` if any file would change. |
| `--stdin-input`, `-i` | Read source from stdin, format it, print the result to stdout. |
| `--stdin-filepath <PATH>` | Logical path of the stdin buffer. Requires `--stdin-input`. Checked against `source.excludes` and `formatter.excludes`; if matched, the input is echoed back unchanged. Also replaces `<stdin>` in diagnostic messages. |
| `--staged`, `-s` | Format only files staged in git and re-stage them. Designed for pre-commit hooks. |
| `-h`, `--help` | Print help and exit. |
