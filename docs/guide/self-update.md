---
title: Updating Mago
---

# Updating Mago

Keeping **Mago** up-to-date is simple. We provide a built-in command to handle the entire update process for you, ensuring you always have the latest features and performance improvements.

## Recommended method

If you installed Mago using the shell script, `brew`, `cargo`, or by manually downloading the binary, the easiest way to update is with the built-in `self-update` command.

```bash
mago self-update
```

This command will:

1.  Check for the latest version of Mago.
2.  Download the new version if it's available.
3.  Replace the current binary with the updated one.

It's the easiest and recommended way to stay current.

## Updating with Composer

If you installed Mago as a project dependency using `composer`, you should also use `composer` to manage its updates. The `self-update` command is not intended for `composer`-managed installations.

To update the `mago` package to the latest version allowed by your `composer.json` constraints, run:

```bash
composer update carthage-software/mago
```

This ensures that your project's `composer.lock` file is updated correctly.

## Checking the current version

After the update process completes, you can verify that you have the latest version by running:

```bash
mago --version
```
