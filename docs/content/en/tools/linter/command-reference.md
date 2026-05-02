+++
title = "Linter command reference"
description = "Every flag mago lint accepts."
nav_order = 50
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Command reference

```sh
Usage: mago lint [OPTIONS] [PATH]...
```

Global flags must come before `lint`. See the [CLI overview](/fundamentals/command-line-interface/) for the global list.

## Arguments

| Argument | Description |
| :--- | :--- |
| `[PATH]...` | Files or directories to lint. When provided, these replace the `paths` from `mago.toml` for this run. |

## Linter-specific options

| Flag | Description |
| :--- | :--- |
| `--list-rules` | List every enabled rule and its description. |
| `--json` | Use with `--list-rules` to emit a machine-readable JSON dump. |
| `--explain <CODE>` | Print detailed documentation for one rule, for example `--explain no-redundant-nullsafe`. |
| `--only <CODE>`, `-o` | Run only the listed rules. Comma-separated. Overrides the config. |
| `--pedantic` | Enable every rule, ignoring PHP-version gates and enabling rules disabled by default. |
| `--semantics`, `-s` | Run parse + semantic check only. Skip the lint rules. |
| `--staged` | Lint only files staged in git. Fails outside a git repository. |
| `--stdin-input` | Read file content from stdin and use the single path argument for baseline lookup and reporting. Intended for editor integrations. |
| `--substitute <ORIG=TEMP>` | Replace one host file with another for this invocation. Intended for mutation testing. Repeatable. |
| `-h`, `--help` | Print help and exit. |

The shared flags for reporting, fixing, and baselines are documented on the [reporting and fixing options](/fundamentals/shared-reporting-options/) page.

## Reading from stdin

When an editor or IDE pipes unsaved buffer content, you can lint that content while still using the real file path for baseline lookup and issue locations:

```sh
cat src/Example.php | mago lint --stdin-input src/Example.php
```

Exactly one path argument is required. It is used as the logical (workspace-relative) file name for baseline matching and diagnostics. The path is normalised, so `./src/Example.php` is treated the same as `src/Example.php`. Conflicts with `--staged`.

## Substituting files

`--substitute ORIG=TEMP` replaces one host file with another for the duration of a single run, without writing anything to disk. Designed for mutation-testing frameworks (Infection and friends) that produce a mutated copy of a source file and want the linter to evaluate the mutation against the rest of the project. If the linter reports a new issue on the mutated file, the mutation can be killed without running the test suite.

```sh
mago lint --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

Rules:

- Both `ORIG` and `TEMP` must be absolute paths and both files must exist.
- `ORIG` must be a host file under one of your configured `paths`. Vendored or excluded files cannot be substituted.
- The flag can be repeated to substitute several files at once.
- Conflicts with `--stdin-input` and `--staged`.

Under the hood, `TEMP` is added to the host paths and `ORIG` is added to the excludes for this run, so cross-file rules continue to see the mutation. Reported issues and baseline entries reference `TEMP` rather than `ORIG`. Mutation-testing tools usually compare issue counts between a clean run and the substituted run, so this does not change the workflow.
