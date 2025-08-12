# Command Line Interface (CLI)

The Command Line Interface (CLI) is the primary way to interact with Mago. It provides a way to run the formatter and linter, as well as other commands.

## Available Commands

### `mago format`

The `format` command is used to format PHP files in your project according to the rules defined in your `mago.toml` configuration file.

- Usage: `mago format [OPTIONS] [PATH]...`
- Arguments:
  - `PATH`: Format specific files or directories, overriding the source configuration.
- Options:
  - `--dry-run`: Preview changes without modifying files.
- Aliases: `mago fmt`

### `mago lint`

The `lint` command is used to analyze PHP files in your project and report any issues found by the linter.

- Usage: `mago lint [OPTIONS]`
- Options:
  - `--fixable-only`: Only show issues that can be automatically fixed.
  - `--semantics-only`: Skip plugin-based rule checks and focus on code correctness.
  - `--reporting-format`: Specify the output format for issue reports (e.g., `rich`, `github`, `json`, `checkstyle`, ...).
  - `--reporting-target`: Specify the target for issue reports (e.g., `stdout`, `stderr` ).

### `mago help`

The `help` command provides information about available commands and their usage.

- Usage: `mago help [COMMAND]`
