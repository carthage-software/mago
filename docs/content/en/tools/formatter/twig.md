+++
title = "Twig templates"
description = "Format Twig syntax without reformatting HTML."
nav_order = 35
nav_section = "Tools"
nav_subsection = "Formatter"
+++
# Twig templates

`mago fmt` formats `.twig` files, including names such as `index.html.twig`. It finds them under `source.paths` without adding `twig` to `source.extensions`. PHP-only tools keep their existing file selection.

```sh
mago fmt templates/
mago fmt --check
mago fmt --dry-run
mago fmt --staged
mago fmt --stdin-input --stdin-filepath templates/index.html.twig < templates/index.html.twig
```

Pass `--stdin-filepath` to select Twig for stdin. Without a path, stdin uses the PHP formatter. The formatter phase of `mago fix` also supports Twig.

## Scope

Mago formats Twig tags, expressions, comments, and indentation. Its spacing follows the [Twig coding standard](https://twig.symfony.com/doc/3.x/coding_standards.html). It keeps each named argument's separator (`name: value` or `name = value`) so formatting does not require a newer Twig version. Macro defaults stay `name = value`.

It does not parse or reformat HTML, CSS, JavaScript, or other template text. It keeps string contents, interpolation text inside strings, whitespace-control markers (`-` and `~`), and `verbatim` bodies intact. It leaves unknown extension tags as written. It also keeps a tag's layout if spacing would turn a name, such as `not`, into an operator. Syntax errors prevent changes to the affected file.

Files that use the legacy `raw` block also stay unchanged.

Mago does not add a final newline or change line endings in template text. `end-of-line` only controls line breaks that the formatter adds inside Twig tags.

Formatting changes what Twig's `source()` function returns for a formatted file. Use `@mago-format-ignore` in files whose source text must stay unchanged.

Mago uses Twig's built-in operators. Use ignore markers in files that rely on custom operators from an extension.

To bound stack use, formatting has a 32-level parser nesting limit. Inputs that exceed it return an error and stay unchanged.

## Indentation

Twig renders whitespace outside its tags as text. Mago keeps every byte of that text unchanged: leading spaces, blank lines, line endings, and the final newline. It only adds indentation inside Twig tags when it wraps code. Wrapped lines follow the tag's existing indentation.

With `print-width = 30`:

```twig
  {{
      user.name|default(
          'A guest with a longer name'
      )
  }}
```

There is no setting that rewrites whitespace outside tags. Tags with inline comments, ambiguous keyword names, or expression trees too deep to format safely stay as written.

## Configuration

Twig settings live under `[formatter.twig]`. They do not inherit PHP formatter settings or presets. `source.excludes` and `formatter.excludes` apply to both languages.

```toml
[formatter.twig]
enabled = true
print-width = 120
tab-width = 4
use-tabs = false
end-of-line = "auto"
```

| Option | Default | Meaning |
| :--- | :--- | :--- |
| `enabled` | `true` | Format `.twig` files. When false, leave them unchanged, including stdin and staged files. |
| `print-width` | `120` | Target width for wrapping Twig expressions. Long strings stay intact. |
| `tab-width` | `4` | Spaces per indentation level and the width of a tab. |
| `use-tabs` | `false` | Use tabs for indentation. |
| `end-of-line` | `"auto"` | Use `auto`, `lf`, `crlf`, or `cr` for generated line breaks. `auto` uses the first line ending in template text outside Twig tags, or LF if there is none. |

To disable Twig formatting:

```toml
[formatter.twig]
enabled = false
```

## Ignore markers

Use the same markers as the PHP formatter, inside Twig comments:

```twig
{# @mago-format-ignore-next #}
{{  leave_this_alone  }}

{# @mago-format-ignore-start #}
{{  leave_this_region_alone  }}
{# @mago-format-ignore-end #}
```

`@mago-format-ignore` skips the whole file. `-next` skips the next Twig statement, including its full body when it opens a block. `@mago-formatter-ignore` and its suffixes work too.
