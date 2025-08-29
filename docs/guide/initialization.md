---
title: Initialization
outline: deep
---

# Initialization

The `mago init` command is the fastest way to set up a new project. It's an interactive walkthrough that creates a `mago.toml` configuration file tailored to your project's needs.

When you run it, Mago will guide you through a series of steps:

```sh
mago init
```

## Configuration paths

1.  If a `composer.json` file is present, Mago will offer to automatically configure your project paths, PHP version, and linter integrations based on its contents. This is the recommended approach for most projects.

2.  If no `composer.json` is found, or if you prefer to set things up manually, the command will prompt you for:
    - Source code paths (`src`, `tests`, etc.)
    - Dependency paths (`vendor`)
    - Paths to exclude
    - PHP version
    - Linter integrations (Symfony, Laravel, etc.)

## Interactive walkthrough

The command also includes an interactive setup for the formatter and analyzer, allowing you to enable powerful features and customize settings right from the start.

Here's an example of what the process looks like:

```sh
$ mago init

 Mago
 ⬩ Welcome! Let's get you set up.

  ╭─ Step 1: Project Setup
  │
  │   Found `composer.json`. Use it to auto-configure your project? › (Y/n)
  │
  │  Reading composer.json...
  │  Project settings detected!
  ╰─
  ╭─ Step 2: Linter Configuration
  │
  │  The Linter checks your code for stylistic issues and inconsistencies.
  │  It helps keep your codebase clean and readable.
  │
  │   Use `composer.json` to auto-detect framework integrations? › (Y/n)
  │
  │  Detecting integrations from composer.json...
  │  Done!
  ╰─

  ... and so on
```

Once the process is complete, you'll have a `mago.toml` file in your project root, and you'll be ready to start analyzing, linting, and formatting your code.
