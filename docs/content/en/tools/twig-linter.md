+++
title = "Twig linter"
description = "Lint Twig templates with native Mago reports, pragmas, and safe fixes."
nav_order = 40
nav_section = "Tools"
nav_subsection = "Twig Linter"
+++
# Twig linter

`mago twig:lint` checks Twig templates. Directory scans select `.twig` files under `source.paths`, including names such as `index.html.twig`, regardless of `source.extensions`. Explicit files, stdin paths, and substitution files use Twig even with no extension or a `.php` extension. This command never uses the PHP parser or PHP rules.

Directory scans and `--staged` match the `.twig` extension with exact case. An explicit `.TWIG` path still works.

```sh
mago twig:lint templates/
mago twig:lint --only twig-no-dump
mago twig:lint --list-rules --json
mago twig:lint --explain twig-no-raw
mago twig:lint --stdin-input templates/page.html.twig < templates/page.html.twig
mago twig:lint templates/ --fix
```

`--semantics` checks Twig syntax only. It does not read lint pragmas or run lint policies or the Twig type analyzer. `--only` uses full `twig-` codes and enables the named rules even if their rule settings disable them. It does not bypass `enabled = false`, exclusions, or a rule's minimum Twig version. `--pedantic` enables every supported rule, including opt-in policies.

The command selects the language; `--only` selects Twig rules. Twig syntax checks still run when no rule matches. `mago lint`, `mago analyze`, `mago fix`, and `mago list-files` keep their own behavior. They do not run Twig lint rules.

Twig issues use Mago's reporting formats, severity threshold, statistics, baselines, substitutions, staged-file handling, and fix safety flags. `--staged` selects staged `.twig` files and stages fixes only when their working copies match the index.

`--substitute ORIG=TEMP` excludes the original and checks the replacement as Twig. Both paths must be absolute. Reports use the replacement path; its extension does not matter. `--stdin-input` requires one path for the report and baseline identity.

## Configuration

```toml
[twig.linter]
enabled = true
version = "3.30"
excludes = ["templates/generated/**"]

[twig.linter.rules.no-dump]
level = "error"
exclude = ["templates/debug/**"]

[twig.linter.rules.variable-name]
enabled = true

[twig.linter.rules.disallowed-functions]
enabled = true
names = ["source"]
```

| Option | Default | Meaning |
| :--- | :--- | :--- |
| `enabled` | `true` | Run `twig:lint`. When false, skip scanning and linting. Other tools are unchanged. |
| `version` | `"3.0"` | The oldest Twig 3 version the project supports, as `3.MINOR`. Set it to enable version-specific rules and fixes. Mago does not infer it from Composer or validate every feature against that version. The parser accepts current Twig syntax. |
| `excludes` | `[]` | Twig-only path prefixes or globs. Mago skips excluded files before parsing them. |
| `baseline` | None | Baseline path relative to `source.workspace`, or an absolute path. `--baseline` overrides it. |
| `baseline-variant` | `"loose"` | Use count-based `loose` entries or line-based `strict` entries when writing a baseline. |
| `minimum-fail-level` | `"error"` | Lowest issue level that fails the command. `--minimum-fail-level` overrides it. |
| `rules.<name>.enabled` | Rule default | Turn a rule on or off. Omit the `twig-` prefix in configuration keys. |
| `rules.<name>.level` | Rule default | Set `note`, `help`, `warning`, or `error`. The usual `twig.linter.minimum-fail-level` controls command failure. |
| `rules.<name>.exclude` | `[]` | Path prefixes or globs for one rule. These use `source.glob` settings and workspace-relative file names. |
| `rules.disallowed-functions.names` | `[]` | Exact function names to disallow, including imported macro aliases. Enable the rule too. |
| `rules.disallowed-filters.names` | `[]` | Exact filter names to disallow. Enable the rule too. |
| `rules.disallowed-tags.names` | `[]` | Exact tag names to disallow, including custom tags. Enable the rule too. |

`source.paths`, `source.excludes`, and `source.glob` are shared file settings. PHP `[linter]` rules, exclusions, baselines, and severity thresholds do not affect `twig:lint`. Configuration layers merge `[twig.linter]` through the usual `extends` rules. Unknown fields and rule names are errors. Use `mago config --show twig.linter` or `mago config --schema --show twig.linter` to inspect them. `config --show linter` remains PHP-only.

`mago config --show twig` shows the configured Twig tools. Omitted tools use their own defaults when run; `mago config --show twig.linter --default` shows the linter's defaults.

## Rules

The [rules page](/tools/twig-linter/rules/) lists every rule with its default level, required Twig version, settings, and examples. Rule codes start with `twig-`; configuration keys leave that prefix out. Rules that are off by default are opt-in policies, not claims that valid Twig syntax is a bug.

Built-in function checks respect imported macro aliases and their lexical scopes. Mago does not treat each unknown custom tag, filter, function, or test as an error. Configured name bans intentionally apply by name, even to imported macros. Naming rules leave non-ASCII names unchanged. Literal-key checks leave dynamic keys, unary string conversions, and out-of-range numeric conversions unknown.

These rules draw on the [official Twig coding standard](https://twig.symfony.com/doc/3.x/coding_standards.html), [Twig-CS-Fixer's rule catalogue](https://twigcsfixer.github.io/rules), and [Twig's source](https://github.com/twigphp/Twig). Mago keeps spacing, indentation, commas, and other layout work in the [Twig formatter](/tools/formatter/twig/). It does not copy text/EOF fixes that can change rendered output.

## Pragmas

Only whitespace and other comments may separate an own-line `{# #}` pragma from its statement. Rendered text or a closing tag ends that scope; the pragma cannot suppress a later statement.

An own-line `#` pragma inside a code tag covers only the rest of that tag. It cannot reach an `if`/block body or a later tag.

Use Twig comments, including inline expression comments where Twig supports them:

```twig
{# @mago-expect lint:twig-no-dump(2) Debug output in this branch is deliberate. #}
{% if debug %}
    {{ dump(first) }}
    {{ dump(second) }}
{% endif %}
```

`lint` and `linter` categories work. Comma-separated codes and repeated category prefixes work too. An own-line comment before a statement covers that statement and its body. A trailing comment covers its own line. Nested comments take precedence. An explicit `(N)` caps matches; an uncounted scoped pragma keeps the usual unlimited-within-scope behavior. `--only` does not report unmet expectations for inactive codes. In a full run, disabled or excluded rules can leave expectations unfulfilled, as in PHP.

Mago reports unused ignores and unfulfilled expectations. Their safe cleanup edits stay inside comment bodies, retain trim markers and delimiters, and keep explicit `(1)` counts. Grouped cleanup can need a second `twig:lint --fix` pass. `mago fix` does not run Twig lint rules.

## Fix safety and limits

Safe fixes change only named separators, missing closing names, conservative string quotes, or pragma bodies. They never rewrite rendered whitespace, string contents, or expression order. Named separator fixes need an explicit minimum version of 3.12 or later. `--potentially-unsafe` and `--unsafe` add no Twig fixes today.

`twig:lint` rejects `--format-after-fix` before scanning or writing. Twig formatting must run as a separate step; this command does not change the current formatter's routing.

Files with legacy `raw` blocks get diagnostics but no edits, at any safety level. Mago keeps the whole file unchanged, including pragma bodies, because legacy or custom `raw` handling can change how Twig reads its bounds.

Mago assumes Twig's built-in lexer delimiters, interpolation syntax, and operators. Exclude files or affected rules when custom lexer settings change those meanings. Unknown-tag payloads and `verbatim` bodies stay opaque. Mago cannot know when an application replaces a built-in function/filter or supplies a custom callable parser; configure the affected rules for that project.

Any source edit changes what Twig's `source()` returns. Exclude such files if their exact source bytes form part of the output contract.

The parser has a 32-level nesting budget; one construct can use more than one parser level. `twig-depth-limit` means Mago did not lint a template that exceeds this budget, not that Twig rejects it. Other parser errors use `twig-syntax-error`, and no rule runs on such a template. Pragmas cannot suppress either code. Long flat expression chains still get linted. The debug library path has tested 4 MiB stacks; smaller caller stacks may not suffice. Mago does not raise the configured worker stack to hide deep-input failures.
