---
title: Analyzer usage
---

# Using the analyzer

The `mago analyze` command is your primary tool for running a deep, type-level analysis on your project.

## Analyzing your project

To analyze all the source files defined in your `mago.toml` configuration, simply run:

```sh
mago analyze
```

Mago will first compile a model of your entire codebase (including dependencies and stubs) and then analyze your source files in parallel, reporting any issues it finds.

## Analyzing specific files

You can also analyze specific files or directories by passing them as arguments. This will override the `paths` in your configuration for this run.

```sh
# Analyze a single file
mago analyze src/Services/UserService.php

# Analyze an entire directory
mago analyze src/Controller/
```

## Working with a baseline

When introducing Mago to an existing project, you might have a large number of pre-existing issues. A "baseline" allows you to ignore these for now and focus only on new issues in new code.

To generate a baseline file:

```sh
mago analyze --generate-baseline
```

This creates a `mago-baseline.php` file. Commit this file, and Mago will automatically ignore all the issues it contains in future runs.

To use a baseline, simply run the analyzer as usual. Mago will automatically detect and use `mago-baseline.php` if it exists. You can also specify a custom path:

```sh
mago analyze --baseline /path/to/your/baseline.php
```

## Disabling stubs

By default, Mago loads a "prelude" of stubs for PHP's built-in functions and classes. If you wish to disable this for any reason, you can use the `--no-stubs` flag. Note that this may lead to a large number of "symbol not found" errors if your code uses standard PHP features.

```sh
mago analyze --no-stubs
```
