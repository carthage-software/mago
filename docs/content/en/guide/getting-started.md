+++
title = "Getting Started"
description = "What Mago is, what it does, and where to go next."
nav_order = 10
nav_section = "Guide"
+++
# Getting Started

Mago is a PHP toolchain written in Rust. One binary covers the parts of a workflow that usually take three or four separate tools.

It includes:

- A [formatter](/tools/formatter/overview/) that produces deterministic output and follows PER-CS by default.
- A [linter](/tools/linter/overview/) with a curated catalogue of rules across nine categories. Many fixes apply automatically.
- A [static analyzer](/tools/analyzer/overview/) that catches type errors and logic bugs before runtime, with support for Psalm and PHPStan annotations.
- An [architectural guard](/tools/guard/overview/) that enforces dependency rules and structural conventions.

The whole thing runs single-binary, with no PHP runtime, no Composer dependency, and no Java install. A typical workflow looks like:

```sh
mago init           # write a starter mago.toml
mago lint           # surface stylistic and correctness issues
mago format         # rewrite files to match the formatter
mago analyze        # type-check and find logic bugs
```

## Where to go next

- [Installation](/guide/installation/) walks through every supported install method.
- [Initialization](/guide/initialization/) covers the interactive `mago init` setup.
- [Configuration](/guide/configuration/) is the reference for every option in `mago.toml`.
- [The playground](/playground/) runs the full Mago analyzer in your browser if you want to try it without installing.
