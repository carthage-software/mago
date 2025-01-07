# Configuration

Mago uses a configuration file named `mago.toml` to define how it discovers, processes, formats,
and lint your project files. This file should be placed in the root directory of your project,
as Mago expects to find it in the same directory you run the tool from.

The configuration options are flexible and allow you to customize paths, exclusions, inclusions,
and rules for formatting and linting.
Below is a detailed but simplified explanation to help you set up your configuration file effectively.

## General Example

Here’s a minimal example of what a mago.toml file might look like:

```toml
[source]
paths = ["src", "tests"]
includes = ["vendor"]
excludes = ["**/src/**/*.generated.php", "tests/fixtures"]
```

## Configuration Options

### Source Configuration

The `[source]` section controls how Mago discovers and processes files. It allows you to define the root directory,
specific paths to scan, additional inclusions, and exclusions.

#### Root Directory

The `root` option defines the base directory for file discovery. If omitted, Mago defaults to the current working directory.
Typically, this is the root of your project.

- Default: `.` (current working directory)
- Type: `string`
- Example:

  ```toml
  root = "/path/to/project"
  ```

If not set, Mago uses the directory where the command is run.

#### Paths

The `paths` option specifies which directories to scan for files. If no paths are defined, Mago will scan the entire root directory.

- Default: `[]`
- Type: `array of strings`
- Example:

  ```toml
  paths = ["src", "tests"]
  ```

Use this to limit Mago’s search scope to specific directories.

#### Includes

The includes option is for adding extra files or directories that are not part of the main source paths but still need to be scanned. This is useful for external dependencies, such as third-party libraries or vendor directories.

- Default: `[]`
- Type: `array of strings`
- Example:

  ```toml
  includes = ["vendor"]
  ```

#### Excludes

The excludes option allows you to define patterns or paths to skip during file discovery. Patterns can include wildcards (\*) for more flexibility.

- Default: `[]`
- Type: `array of strings`
- Example:

  ```toml
  excludes = ["tests/fixtures", "**/src/**/*.generated.php"]
  ```

#### Extensions

The extensions option specifies the file extensions to include in the search.

- Default: `["php"]`
- Type: `array of strings`
- Example:

  ```toml
  extensions = ["php", "php8"]
  ```

### Formatter Configuration

The `[formatter]` section customizes how Mago formats your PHP code, including settings like line width, tab width, and indentation style.

For more details on the available formatter settings, see the [Formatter Settings](/formatter/settings.md) page.

- Default: `{}`
- Type: `table`
- Example:

  ```toml
  [formatter]
  print_width = 80
  tab_width = 2
  ```

### Linter Configuration

The `[linter]` section controls the behavior of Mago's linter. This includes enabling plugins,
setting rule configurations, and determining the level of severity for specific rules.

Below is a detailed explanation of the available options.

#### Default Plugins

This setting specifies whether to enable the default set of linter plugins.

- Default: `true`
- Type: `boolean`
- Example:

  ```toml
  [linter]
  default_plugins = false
  ```

#### Plugins

The `plugins` option allows you to specify which linter plugins to enable.

- Default: `[]`
- Type: `array of strings`
- Example:

  ```toml
  [linter]
  plugins = ["symfony", "laravel", "phpunit"]
  ```

#### Rules

The `rules` option lets you configure specific rules for the linter. Each rule can have its own severity level, as well as additional options.

Each rule is specified as an array entry under `[[linter.rules]]`, with the following properties:

- `name`: The name of the rule in `{plugin}/{rule}` format.
- `level`: The severity level of the rule (`"error"`, `"warning"`, `"info"`, `"help"`, or `"off"`).
- `...`: Additional options specific to the rule.

Here's an example of configuring a rule:

```toml
[[linter.rules]]
name = "best-practices/excessive-nesting"
level = "warning"
threshold = 5

[[linter.rules]]
name = "safety/no-ffi"
level = "off"
```

For more information on the available plugins and rules, see the [Linter Plugins](/linter/plugins/) page.
