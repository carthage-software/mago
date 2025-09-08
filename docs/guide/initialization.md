---
title: "Initialization"
---

# Initialization

The `mago init` command is the fastest way to get started with Mago on a new or existing project. Its purpose is to generate a `mago.toml` configuration file by guiding you through an interactive setup process.

This command intelligently detects your project's structure (e.g., by reading `composer.json`) to provide sensible defaults, but also gives you the flexibility to customize every detail.

By the end of the process, you will have a configuration file tailored to your project, ready for you to start using Mago's powerful tools.

## Usage

To begin configuring your project, simply run the `init` command from your project's root directory:

```sh
mago init
```

This will start an interactive walkthrough. Mago will ask you a series of questions about your project to generate the best configuration for your needs.

### Automatic detection with `composer.json`

If a `composer.json` file is present, Mago will offer to automatically configure your project paths, PHP version, and linter integrations based on its contents. This is the recommended approach for most projects.

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
```

### Manual setup

If no `composer.json` is found, or if you choose to configure things manually, the command will prompt you for:

- Source code paths (`src`, `tests`, etc.)
- Dependency paths (`vendor`)
- Paths to exclude
- PHP version
- Linter integrations (Symfony, Laravel, etc.)

Once the process is complete, you'll have a `mago.toml` file in your project root. For more details on the available options in this file, see the [Configuration Overview](/guide/configuration.md).

## Command reference

:::tip
For global options that can be used with any command, see the [Command-Line Interface overview](/fundamentals/command-line-interface.md). Remember to specify global options **before** the `init` command.
:::

```sh
Usage: mago init [OPTIONS]
```

### Options

| Flag, Alias(es) | Description             |
| :-------------- | :---------------------- |
| `-h`, `--help`  | Print help information. |
