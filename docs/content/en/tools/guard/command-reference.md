+++
title = "Guard command reference"
description = "Every flag mago guard accepts."
nav_order = 30
nav_section = "Tools"
nav_subsection = "Guard"
+++
# Command reference

```sh
Usage: mago guard [OPTIONS] [PATHS]...
```

Global flags must come before `guard`. See the [CLI overview](/fundamentals/command-line-interface/) for the global list.

## Arguments

| Argument | Description |
| :--- | :--- |
| `[PATHS]...` | Files or directories to check. When provided, these replace the `paths` from `mago.toml` for this run. |

## Mode selection

These flags pick which half of the guard runs. They are mutually exclusive.

| Flag | Description |
| :--- | :--- |
| `--structural` | Run only structural checks (naming, modifiers, inheritance). |
| `--perimeter` | Run only perimeter checks (dependency boundaries, layer restrictions). |

If neither flag is set, both halves run, the same as `mode = "default"` in configuration. These flags override the configured `mode`. If the flag matches the configured mode, the guard prints a redundancy warning.

## Other options

| Flag | Description |
| :--- | :--- |
| `--no-stubs` | Skip the built-in PHP and library stubs. Use only when you have a reason. |
| `--stdin-input` | Read file content from stdin and use the single path argument for baseline lookup and reporting. Intended for editor integrations. |
| `--substitute <ORIG=TEMP>` | Replace one host file with another for this invocation. Intended for mutation testing. Repeatable. |
| `-h`, `--help` | Print help and exit. |

The shared flags for reporting, fixing, and baselines are documented on the [reporting and fixing options](/fundamentals/shared-reporting-options/) page. Auto-fixing is not currently meaningful for guard issues, but the flags are accepted for parity with the other tools.

## Reading from stdin

For editor integrations that pipe unsaved buffer content:

```sh
cat src/Example.php | mago guard --stdin-input src/Example.php
```

Exactly one path argument is required. It is used as the workspace-relative file name for baseline matching and diagnostics. The path is normalised, so `./src/Example.php` is treated the same as `src/Example.php`. Conflicts with `--substitute`.

## Substituting files

`--substitute ORIG=TEMP` replaces one host file with another for the duration of a single run without writing anything to disk. Designed for mutation-testing frameworks that produce a mutated copy of a source file and want the guard to evaluate the mutation against the rest of the project.

```sh
mago guard --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

Rules:

- Both `ORIG` and `TEMP` must be absolute paths and both files must exist.
- `ORIG` must be a host file under one of your configured `paths`. Vendored or excluded files cannot be substituted.
- The flag can be repeated to substitute several files at once.
- Conflicts with `--stdin-input`.

Under the hood, `TEMP` is added to host paths and `ORIG` is added to excludes for this run, so dependency analysis continues to see the mutation. Reported issues reference `TEMP` rather than `ORIG`.
