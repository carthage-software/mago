---
title: Listing files
---

# Listing scanned files

Sometimes it might be useful to get a list of all files that a given Mago
command would scan given the current configuration. For example to use it to
refine the current configuration, or to pass it to a different tool for some
kind of processing. This can be done using the list-files command.

## Usage

To get a list of all files configured as sources, simply run the `list-files` command:

```sh
mago list-files
```

As Mago's command can have their own file exclusions, the set of files might
differ between the commands. If you want to see which files a certain command
would process, use the `--command` option:

```sh
mago list-files --command linter
```

As filenames may contain newlines, the default of printing one name per line is prone to errors when passing the list to other tools like `xargs`. In that case, you can have the filenames be zero-terminated instead:

```sh
mago list-files -0 | xargs -0r ls -l
```

## Command reference

:::tip
For global options that can be used with any command, see the [Command-Line
Interface overview](/fundamentals/command-line-interface.md). Remember to
specify global options **before** the `list-files` command.
:::

```sh
Usage: mago list-files [OPTIONS]
```

### Options

| Flag, Alias(es)           | Description                                                                                                    |
| :------------------------ | :------------------------------------------------------------------------------------------------------------- |
|       `--command`         | Select for which command the file list should be generated. <br/>**Values:** `linter`, `formatter`, `analyzer`, `guard` |
| `-0`, `--zero-terminate`  | Use NUL bytes instead of newlines to terminate the filenames.                                                  |
| `-h`, `--help`            | Print help information.                                                                                        |