//! Configuration management for Mago CLI.
//!
//! Configuration is loaded by reading at most one config file (TOML/YAML/JSON), recursively
//! resolving its `extends` chain, merging the layers, and finally applying environment
//! variable + CLI overrides for the small set of top-level scalars.
//!
//! # File discovery
//!
//! - When `--config <path>` is given, that exact file is loaded. Format is detected from the
//!   extension; unrecognised extensions are an error.
//! - Otherwise, the loader looks for `mago.{toml,yaml,yml,json}` (then `mago.dist.{...}`) in:
//!   1. The workspace directory.
//!   2. `$XDG_CONFIG_HOME`.
//!   3. `~/.config`.
//!   4. `~`.
//!   The first match wins; workspace beats global. If nothing is found, built-in defaults
//!   are used.
//! - Within a directory, format precedence is `toml > yaml > yml > json`.
//!
//! # `extends`
//!
//! A config file may opt to inherit from one or more other files via a top-level `extends`
//! directive:
//!
//! ```toml
//! extends = "vendor/some-org/some-pkg/mago.toml"        # single file
//! extends = ["base.toml", "configs/strict.json"]         # multiple, applied in order
//! extends = ["../shared"]                                # directory: looks for mago.{toml,yaml,yml,json}
//! ```
//!
//! - Paths are absolute or relative; relative paths resolve **against the directory of the
//!   file declaring the `extends`**, not against cwd. This is important when `--config
//!   path/to/foo.toml` declares `extends = "bar.toml"` — `bar.toml` is resolved next to
//!   `foo.toml`.
//! - File entries must exist and have a recognised extension.
//! - Directory entries are searched for a `mago.{toml,yaml,yml,json}` file; if none is
//!   found, the entry is skipped with a warning.
//! - Cycles are detected via canonical-path tracking and surface as a clean error.
//!
//! # Merge semantics
//!
//! Layers are merged later-wins. For each top-level key:
//! - **Tables / objects**: deep-merged recursively.
//! - **Arrays** (e.g. `source.excludes`): concatenated, parent first.
//! - **Scalars**: child overwrites parent.
//!
//! All layers parse into a generic `serde_json::Value` tree. The merged tree is deserialized
//! into [`Configuration`] exactly once, so schema validation runs a single time regardless
//! of chain depth.
//!
//! # Effective precedence (lowest → highest)
//!
//! 1. Built-in defaults (via `#[serde(default)]` on every field).
//! 2. Each layer reachable through `extends`, applied in declared order; transitively
//!    each parent's own `extends` is fully resolved before the parent's keys apply.
//! 3. The owning file's own keys.
//! 4. Environment variables (top-level scalars only — see below).
//! 5. CLI overrides (e.g. `--php-version`, `--threads`).
//!
//! # Environment variables
//!
//! Mago officially recognises only the following top-level overrides. Anything else
//! prefixed `MAGO_` is reserved for internal use and may be silently ignored:
//!
//! - `MAGO_PHP_VERSION` → `php-version`
//! - `MAGO_THREADS` → `threads`
//! - `MAGO_STACK_SIZE` → `stack-size`
//! - `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION` → `allow-unsupported-php-version`
//! - `MAGO_NO_VERSION_CHECK` → `no-version-check`
//! - `MAGO_EDITOR_URL` → `editor-url`
//!
//! For anything deeper, edit the config file.
//!
//! # Normalization and validation
//!
//! After loading, the configuration is normalized:
//!
//! - Thread count defaults to logical CPU count if zero.
//! - Stack size is clamped between minimum and maximum bounds.
//! - PHP version compatibility is validated against the supported range.
//! - Source paths are resolved and validated.

use std::collections::HashSet;
use std::env::home_dir;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

// Note: format detection + parsing is handled by `ConfigFormat` below; we no longer route
// through the `config` crate, which added ~1.5ms of intermediate-Value-tree overhead.
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use serde_json::Value;

use crate::config::analyzer::AnalyzerConfiguration;
use crate::config::formatter::FormatterConfiguration;
use crate::config::guard::GuardConfiguration;
use crate::config::linter::LinterConfiguration;
use crate::config::parser::ParserConfiguration;
use crate::config::source::SourceConfiguration;
use crate::consts::*;
use crate::error::Error;

pub mod analyzer;
pub mod formatter;
pub mod guard;
pub mod linter;
pub mod parser;
pub mod source;

/// Default value for threads configuration field.
fn default_threads() -> usize {
    *LOGICAL_CPUS
}

/// Default value for stack_size configuration field.
fn default_stack_size() -> usize {
    DEFAULT_STACK_SIZE
}

/// Default value for php_version configuration field.
fn default_php_version() -> PHPVersion {
    DEFAULT_PHP_VERSION
}

/// Default value for source configuration field.
fn default_source_configuration() -> SourceConfiguration {
    SourceConfiguration::from_workspace(CURRENT_DIR.clone())
}

// Pre-baked env-var names. Keeps the prefix in sync with `ENVIRONMENT_PREFIX` (see assertion
// below) without paying for `format!` allocations on every load.
const ENV_PHP_VERSION: &str = "MAGO_PHP_VERSION";
const ENV_THREADS: &str = "MAGO_THREADS";
const ENV_STACK_SIZE: &str = "MAGO_STACK_SIZE";
const ENV_ALLOW_UNSUPPORTED_PHP_VERSION: &str = "MAGO_ALLOW_UNSUPPORTED_PHP_VERSION";
const ENV_NO_VERSION_CHECK: &str = "MAGO_NO_VERSION_CHECK";
const ENV_EDITOR_URL: &str = "MAGO_EDITOR_URL";

const _: () = {
    // Compile-time guard: if anyone changes `ENVIRONMENT_PREFIX`, this file must be updated
    // too. We do a byte-by-byte comparison since `&str` equality isn't const-stable yet.
    let bytes = ENVIRONMENT_PREFIX.as_bytes();

    assert!(bytes.len() == 4 && bytes[0] == b'M' && bytes[1] == b'A' && bytes[2] == b'G' && bytes[3] == b'O');
};

/// The main configuration structure for Mago CLI.
///
/// Aggregates all settings: top-level scalars (`php-version`, `threads`, …) plus
/// service-specific sub-configurations (linter, analyzer, formatter, guard, parser).
///
/// Loaded by [`Configuration::load`]; see the [module-level documentation](self) for the
/// full precedence order and `extends` semantics. Strict validation via
/// `deny_unknown_fields` catches typos early (note that the `extends` directive itself is
/// stripped from each layer before deserialization, so it never trips the strict check).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Configuration {
    /// The mago version this project is pinned to.
    ///
    /// Accepts three pin levels:
    ///
    /// - `"1"`: major pin; any `1.x.y` satisfies it. `mago init` emits this
    ///   by default. Minor/patch drift within major 1 is a warning; a bump to
    ///   `2.x` is a hard error.
    /// - `"1.19"`: minor pin; any `1.19.y` satisfies it.
    /// - `"1.19.3"`: exact pin; any drift is a warning, and this is the only
    ///   form that `mago self-update --to-project-version` can target without
    ///   ambiguity.
    ///
    /// Empty / missing is currently a no-op; a future mago release is likely
    /// to start warning when the pin is absent, to prepare projects for 2.0.
    ///
    /// # Compatibility invariant (**do not break**)
    ///
    /// This field's location (top-level) and type (string) are a load-bearing
    /// contract for cross-major config compatibility: a future mago 2.x must
    /// be able to read a mago 1.x `mago.toml` via a permissive top-level TOML
    /// pass, find this field, and refuse to run with
    /// "this config is pinned to mago 1" *before* it ever hits its own strict
    /// schema. That means:
    ///
    /// - never move this field into a `[metadata]` section,
    /// - never rename it,
    /// - never change it from a string,
    /// - never change the pin grammar from `major[.minor[.patch]]`.
    #[serde(default)]
    pub version: Option<String>,

    /// Number of worker threads for parallel processing.
    ///
    /// Controls the thread pool size used by Rayon for parallel operations.
    /// If set to 0, defaults to the number of logical CPUs available.
    /// Can be overridden via `MAGO_THREADS` environment variable or `--threads` CLI flag.
    #[serde(default = "default_threads")]
    pub threads: usize,

    /// Stack size for each worker thread in bytes.
    ///
    /// Determines the maximum stack size allocated for each thread in the thread pool.
    /// Must be between `MINIMUM_STACK_SIZE` and `MAXIMUM_STACK_SIZE`.
    /// If set to 0, uses `MAXIMUM_STACK_SIZE`. Values outside the valid range are
    /// automatically clamped during normalization.
    #[serde(default = "default_stack_size")]
    pub stack_size: usize,

    /// Target PHP version for parsing and analysis.
    ///
    /// Specifies which PHP version to assume when parsing code and performing analysis.
    /// This affects syntax parsing rules, available built-in functions, and type checking.
    /// Can be overridden via `MAGO_PHP_VERSION` environment variable or `--php-version` CLI flag.
    #[serde(default = "default_php_version")]
    pub php_version: PHPVersion,

    /// Whether to allow PHP versions not officially supported by Mago.
    ///
    /// When enabled, Mago will not fail if the specified PHP version is outside the
    /// officially supported range. Use with caution as behavior may be unpredictable.
    /// Can be enabled via `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION` environment variable
    /// or `--allow-unsupported-php-version` CLI flag.
    #[serde(default)]
    pub allow_unsupported_php_version: bool,

    /// Whether to silence the project version drift warning.
    ///
    /// Affects only the minor / patch drift warning emitted when the installed
    /// mago binary does not match the `version` pinned in `mago.toml`. A major
    /// drift is always fatal and is *not* affected by this flag; the whole
    /// point of a major pin is to stop runs across incompatible config schemas.
    ///
    /// Can be enabled via `MAGO_NO_VERSION_CHECK` environment variable or
    /// `--no-version-check` CLI flag.
    #[serde(default)]
    pub no_version_check: bool,

    /// Source discovery and workspace configuration.
    ///
    /// Defines the workspace root, source paths to scan, and exclusion patterns.
    /// This configuration determines which PHP files are loaded into the database
    /// for analysis, linting, or formatting.
    #[serde(default = "default_source_configuration")]
    pub source: SourceConfiguration,

    /// Linter service configuration.
    ///
    /// Controls linting behavior, including enabled/disabled rules, rule-specific
    /// settings, and reporting preferences. Defaults to an empty configuration
    /// if not specified in the config file.
    #[serde(default)]
    pub linter: LinterConfiguration,

    /// Parser configuration.
    ///
    /// Controls how PHP code is parsed, including lexer-level settings
    /// like short open tag support. Defaults to standard PHP parsing behavior
    /// if not specified in the config file.
    #[serde(default)]
    pub parser: ParserConfiguration,

    /// Formatter service configuration.
    ///
    /// Defines code formatting style preferences such as indentation, line width,
    /// brace placement, and spacing rules. Defaults to standard formatting settings
    /// if not specified in the config file.
    #[serde(default)]
    pub formatter: FormatterConfiguration,

    /// Type analyzer service configuration.
    ///
    /// Controls static type analysis behavior, including strictness levels,
    /// inference settings, and type-related rules. Defaults to an empty configuration
    /// if not specified in the config file.
    #[serde(default)]
    pub analyzer: AnalyzerConfiguration,

    /// Guard service configuration for continuous monitoring.
    ///
    /// Defines settings for the guard/watch mode, including file watching behavior,
    /// debouncing, and incremental analysis. Defaults to an empty configuration
    /// if not specified in the config file.
    #[serde(default)]
    pub guard: GuardConfiguration,

    /// Editor URL template for OSC 8 terminal hyperlinks on file paths in diagnostics.
    ///
    /// When set, file paths in diagnostic output become clickable links in terminals
    /// that support OSC 8 hyperlinks (e.g., iTerm2, Wezterm, Kitty, Windows Terminal).
    ///
    /// Supported placeholders:
    /// - `%file%` — absolute file path
    /// - `%line%` — line number
    /// - `%column%` — column number
    ///
    /// Can be set via `MAGO_EDITOR_URL` environment variable or `editor-url` in `mago.toml`.
    #[serde(default)]
    pub editor_url: Option<String>,

    /// The path to the configuration file that was loaded, if any.
    ///
    /// This is set during configuration loading and is not user-configurable.
    /// It is used by watch mode to monitor the configuration file for changes.
    #[serde(default, skip_serializing)]
    #[schemars(skip)]
    pub config_file: Option<PathBuf>,

    /// Whether the configuration file was explicitly provided via `--config` CLI flag.
    ///
    /// When `true`, the config file path is pinned and won't be re-discovered on reload.
    /// When `false`, the config file is auto-discovered and may change between reloads.
    #[serde(default, skip_serializing)]
    #[schemars(skip)]
    pub config_file_is_explicit: bool,
}

impl Configuration {
    /// Locate, parse, merge (`extends` chain), and normalize the configuration.
    ///
    /// See the [module-level documentation](self) for the full file-discovery, `extends`,
    /// merge, and precedence rules. In short:
    ///
    /// 1. Pick the entry config file: explicit `--config` if given, else search for one.
    /// 2. Recursively load it: each layer's `extends` is resolved (relative to that file's
    ///    directory) and merged before the layer's own keys apply.
    /// 3. Deserialize the merged document into [`Configuration`] once.
    /// 4. Apply env-var overrides for the top-level scalars (`MAGO_PHP_VERSION`, …).
    /// 5. Apply the explicit CLI overrides passed in here.
    /// 6. Normalize (clamp, validate, fill defaults).
    ///
    /// # Arguments
    ///
    /// * `workspace` — workspace directory; defaults to cwd.
    /// * `file` — explicit config file (`--config`); when given, fallback search is skipped.
    /// * `php_version` — CLI override for `php-version`.
    /// * `threads` — CLI override for `threads`.
    /// * `allow_unsupported_php_version` — only forces `true`; never forces `false`.
    /// * `no_version_check` — only forces `true`; never forces `false`. Does not affect
    ///   the fatal behaviour on major-version drift.
    ///
    /// # Errors
    ///
    /// - `ReadConfigFile` / `ParseConfigFile` — I/O or syntax failures on any layer.
    /// - `UnsupportedConfigExtension` — explicit file or `extends` entry has a non-recognised extension.
    /// - `CircularExtends` — `extends` chain visits the same canonical file twice.
    /// - `ExtendsTargetNotFound` — `extends` entry does not exist on disk.
    /// - `InvalidExtendsEntry` — `extends` is not a string or array of strings.
    /// - `EnvVarParse` — an `MAGO_*` env var is set but its value can't be parsed.
    /// - Normalization errors (invalid source paths, etc.).
    pub fn load(
        workspace: Option<PathBuf>,
        file: Option<&Path>,
        php_version: Option<PHPVersion>,
        threads: Option<usize>,
        allow_unsupported_php_version: bool,
        no_version_check: bool,
    ) -> Result<Configuration, Error> {
        let workspace_dir = workspace.clone().unwrap_or_else(|| CURRENT_DIR.to_path_buf());

        let resolved_config_file: Option<(PathBuf, ConfigFormat)>;
        let config_file_is_explicit;
        if let Some(file) = file {
            tracing::debug!("Sourcing configuration from {}.", file.display());

            resolved_config_file = Some((
                file.to_path_buf(),
                ConfigFormat::for_path(file).ok_or_else(|| Error::UnsupportedConfigExtension(file.to_path_buf()))?,
            ));

            config_file_is_explicit = true;
        } else {
            let fallback_roots = [
                std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
                home_dir().map(|h| h.join(".config")),
                home_dir(),
            ];

            resolved_config_file = Self::find_config_files(&workspace_dir, &fallback_roots);
            if let Some((config_file, _)) = &resolved_config_file {
                tracing::debug!("Sourcing configuration from {}.", config_file.display());
            } else {
                tracing::debug!("No configuration file found, using defaults and environment variables.");
            }

            config_file_is_explicit = false;
        }

        let mut configuration: Configuration = if let Some((path, format)) = &resolved_config_file {
            let mut visited: HashSet<PathBuf> = HashSet::new();
            let merged = load_layer(path, *format, &mut visited)?;
            serde_json::from_value::<Configuration>(merged)
                .map_err(|e| Error::ParseConfigFile { path: path.clone(), source: Box::new(e) })?
        } else {
            Configuration::from_workspace(workspace_dir.clone())
        };

        configuration.apply_env_overrides()?;

        configuration.config_file = resolved_config_file.as_ref().map(|(p, _)| p.clone());
        configuration.config_file_is_explicit = config_file_is_explicit;

        if allow_unsupported_php_version && !configuration.allow_unsupported_php_version {
            tracing::warn!("Allowing unsupported PHP versions.");

            configuration.allow_unsupported_php_version = true;
        }

        if no_version_check && !configuration.no_version_check {
            tracing::info!("Silencing project version drift warning.");

            configuration.no_version_check = true;
        }

        if let Some(php_version) = php_version {
            tracing::info!("Overriding PHP version with {}.", php_version);

            configuration.php_version = php_version;
        }

        if let Some(threads) = threads {
            tracing::info!("Overriding thread count with {}.", threads);

            configuration.threads = threads;
        }

        if let Some(workspace) = workspace {
            tracing::info!("Overriding workspace directory with {}.", workspace.display());

            configuration.source.workspace = workspace;
        }

        if configuration.editor_url.is_none() {
            configuration.editor_url = detect_editor_url();
        }

        configuration.normalize()?;

        Ok(configuration)
    }

    /// Searches for configuration files in a project directory, falling back to global config locations.
    ///
    /// This function attempts to load at most one configuration file per supported config type.
    /// It first searches the given `root_dir` (typically the workspace/project directory).
    /// If any configuration file is found there, those results are returned immediately and no
    /// fallback locations are checked.
    ///
    /// If no config files are found in `root_dir`, the function searches each directory in
    /// `fallback_roots` in order. The first matching format for each config file name is returned.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - The primary directory to search (project root)
    /// * `fallback_roots` - A list of additional directories to search if `root_dir` contains no matches
    /// * `file_formats` - Supported configuration formats (`toml`, `yaml`, etc.), each with possible extensions
    ///
    /// # Returns
    ///
    /// A vector of `(PathBuf, FileFormat)` pairs, where:
    /// * The path is the resolved config file
    /// * The format indicates which file format identified it
    ///
    /// # Behavior Summary
    ///
    /// 1. Try to resolve `<root_dir>/<name>.<ext>` for each supported format
    /// 2. Stop and return immediately if any matches are found
    /// 3. Otherwise, search each directory in `fallback_roots` in order
    /// 4. The first match (by name and directory order) wins
    ///
    /// This prevents global configuration files from overriding project-local configuration.
    fn find_config_files(root_dir: &Path, fallback_roots: &[Option<PathBuf>]) -> Option<(PathBuf, ConfigFormat)> {
        let config_files = [CONFIGURATION_FILE_NAME, CONFIGURATION_DIST_FILE_NAME];

        for name in config_files.iter() {
            let mut candidate = root_dir.join(name);
            // The base path has no extension yet, so the first `set_extension` adds it
            // rather than replacing — equivalent to `with_added_extension(ext)`.
            for format in ConfigFormat::ALL.iter() {
                for ext in format.extensions() {
                    candidate.set_extension(ext);
                    if candidate.exists() {
                        return Some((candidate, *format));
                    }
                }
            }
        }

        for root in fallback_roots.iter().flatten() {
            let mut candidate = root.join(CONFIGURATION_FILE_NAME);
            for format in ConfigFormat::ALL.iter() {
                for ext in format.extensions() {
                    candidate.set_extension(ext);
                    if candidate.exists() {
                        return Some((candidate, *format));
                    }
                }
            }
        }

        None
    }

    /// Apply environment-variable overrides for the small set of documented top-level
    /// scalar fields. We do **not** auto-map nested keys — anyone who needs to override
    /// nested settings does so via the config file. The supported variables are:
    ///
    /// - `MAGO_PHP_VERSION` — overrides `php-version`
    /// - `MAGO_THREADS` — overrides `threads`
    /// - `MAGO_STACK_SIZE` — overrides `stack-size`
    /// - `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION` — overrides `allow-unsupported-php-version`
    /// - `MAGO_NO_VERSION_CHECK` — overrides `no-version-check`
    /// - `MAGO_EDITOR_URL` — overrides `editor-url`
    fn apply_env_overrides(&mut self) -> Result<(), Error> {
        // Env-var names are static — no per-call allocation.
        if let Ok(v) = std::env::var(ENV_PHP_VERSION) {
            self.php_version =
                v.parse().map_err(|e| Error::EnvVarParse { name: ENV_PHP_VERSION, source: Box::new(e) })?;
        }
        if let Ok(v) = std::env::var(ENV_THREADS) {
            self.threads = v.parse().map_err(|e| Error::EnvVarParse { name: ENV_THREADS, source: Box::new(e) })?;
        }
        if let Ok(v) = std::env::var(ENV_STACK_SIZE) {
            self.stack_size =
                v.parse().map_err(|e| Error::EnvVarParse { name: ENV_STACK_SIZE, source: Box::new(e) })?;
        }
        if let Ok(v) = std::env::var(ENV_ALLOW_UNSUPPORTED_PHP_VERSION) {
            self.allow_unsupported_php_version = parse_bool(&v)
                .map_err(|e| Error::EnvVarParse { name: ENV_ALLOW_UNSUPPORTED_PHP_VERSION, source: Box::new(e) })?;
        }
        if let Ok(v) = std::env::var(ENV_NO_VERSION_CHECK) {
            self.no_version_check =
                parse_bool(&v).map_err(|e| Error::EnvVarParse { name: ENV_NO_VERSION_CHECK, source: Box::new(e) })?;
        }
        if let Ok(v) = std::env::var(ENV_EDITOR_URL) {
            self.editor_url = Some(v);
        }
        Ok(())
    }

    /// Creates a default configuration anchored to a specific workspace directory.
    ///
    /// This constructor initializes a configuration with sensible defaults suitable
    /// for immediate use. All service-specific configurations (linter, formatter,
    /// analyzer, guard) use their default settings.
    ///
    /// # Default Values
    ///
    /// - **threads**: Number of logical CPUs available on the system
    /// - **stack_size**: `DEFAULT_STACK_SIZE` (typically 8MB)
    /// - **php_version**: `DEFAULT_PHP_VERSION` (latest stable PHP version)
    /// - **allow_unsupported_php_version**: `false`
    /// - **source**: Workspace-specific source configuration
    /// - **linter**: Default linter configuration
    /// - **formatter**: Default formatter configuration
    /// - **analyzer**: Default analyzer configuration
    /// - **guard**: Default guard configuration
    ///
    /// This method is primarily used as the starting point for configuration loading,
    /// with values subsequently overridden by config files and environment variables.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The root directory of the workspace to analyze. This directory
    ///   serves as the base path for relative source paths and is where the database
    ///   will look for PHP files.
    ///
    /// # Returns
    ///
    /// A new `Configuration` instance with default values and the specified workspace.
    pub fn from_workspace(workspace: PathBuf) -> Self {
        Self {
            version: None,
            threads: *LOGICAL_CPUS,
            stack_size: DEFAULT_STACK_SIZE,
            php_version: DEFAULT_PHP_VERSION,
            allow_unsupported_php_version: false,
            no_version_check: false,
            source: SourceConfiguration::from_workspace(workspace),
            linter: LinterConfiguration::default(),
            parser: ParserConfiguration::default(),
            formatter: FormatterConfiguration::default(),
            analyzer: AnalyzerConfiguration::default(),
            guard: GuardConfiguration::default(),
            editor_url: None,
            config_file: None,
            config_file_is_explicit: false,
        }
    }
}

impl Configuration {
    /// Returns a filtered version of the configuration suitable for display.
    ///
    /// This method excludes linter rules that don't match the configured integrations,
    /// so that only applicable rules are shown in the output.
    #[must_use]
    pub fn to_filtered_value(&self) -> Value {
        serde_json::json!({
            "version": self.version,
            "threads": self.threads,
            "stack-size": self.stack_size,
            "php-version": self.php_version,
            "allow-unsupported-php-version": self.allow_unsupported_php_version,
            "no-version-check": self.no_version_check,
            "source": self.source,
            "linter": self.linter.to_filtered_value(self.php_version),
            "parser": self.parser,
            "formatter": self.formatter.to_value(),
            "analyzer": self.analyzer,
            "guard": self.guard,
        })
    }

    /// Normalizes and validates configuration values.
    ///
    /// This method ensures that all configuration values are within acceptable ranges
    /// and makes sensible adjustments where needed. It is called automatically by
    /// [`load`](Self::load) after merging all configuration sources.
    ///
    /// # Normalization Rules
    ///
    /// ## Thread Count
    ///
    /// - If set to `0`: Defaults to the number of logical CPUs
    /// - Otherwise: Uses the configured value as-is
    ///
    /// ## Stack Size
    ///
    /// - If set to `0`: Uses `MAXIMUM_STACK_SIZE`
    /// - If greater than `MAXIMUM_STACK_SIZE`: Clamped to `MAXIMUM_STACK_SIZE`
    /// - If less than `MINIMUM_STACK_SIZE`: Clamped to `MINIMUM_STACK_SIZE`
    /// - Otherwise: Uses the configured value as-is
    ///
    /// ## Source Configuration
    ///
    /// Delegates to [`SourceConfiguration::normalize`] to validate workspace paths
    /// and resolve relative paths.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if normalization succeeded
    /// - `Err(Error)` if source normalization failed (e.g., invalid workspace path)
    ///
    /// # Side Effects
    ///
    /// This method logs warnings and informational messages when values are adjusted,
    /// helping users understand how their configuration was interpreted.
    fn normalize(&mut self) -> Result<(), Error> {
        match self.threads {
            0 => {
                tracing::info!("Thread configuration is zero, using the number of logical CPUs: {}.", *LOGICAL_CPUS);

                self.threads = *LOGICAL_CPUS;
            }
            _ => {
                tracing::debug!("Configuration specifies {} threads.", self.threads);
            }
        }

        match self.stack_size {
            0 => {
                tracing::info!(
                    "Stack size configuration is zero, using the maximum size of {} bytes.",
                    MAXIMUM_STACK_SIZE
                );

                self.stack_size = MAXIMUM_STACK_SIZE;
            }
            s if s > MAXIMUM_STACK_SIZE => {
                tracing::warn!(
                    "Stack size configuration is too large, reducing to maximum size of {} bytes.",
                    MAXIMUM_STACK_SIZE
                );

                self.stack_size = MAXIMUM_STACK_SIZE;
            }
            s if s < MINIMUM_STACK_SIZE => {
                tracing::warn!(
                    "Stack size configuration is too small, increasing to minimum size of {} bytes.",
                    MINIMUM_STACK_SIZE
                );

                self.stack_size = MINIMUM_STACK_SIZE;
            }
            _ => {
                tracing::debug!("Configuration specifies a stack size of {} bytes.", self.stack_size);
            }
        }

        self.source.normalize()?;

        if let Some(b) = self.analyzer.baseline.take() {
            let resolved = if b.is_relative() { self.source.workspace.join(&b) } else { b };
            tracing::debug!("Analyzer baseline configuration from {}.", resolved.display());
            self.analyzer.baseline = Some(resolved);
        }

        if let Some(b) = self.linter.baseline.take() {
            let resolved = if b.is_relative() { self.source.workspace.join(&b) } else { b };
            tracing::debug!("Linter baseline configuration from {}.", resolved.display());
            self.linter.baseline = Some(resolved);
        }

        if let Some(b) = self.guard.baseline.take() {
            let resolved = if b.is_relative() { self.source.workspace.join(&b) } else { b };
            tracing::debug!("Guard baseline configuration from {}.", resolved.display());
            self.guard.baseline = Some(resolved);
        }

        Ok(())
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use core::str;
    use std::fs;

    use pretty_assertions::assert_eq;
    use tempfile::env::temp_dir;

    use super::*;

    #[test]
    fn test_take_defaults() {
        let workspace_path = temp_dir().join("workspace-0");
        std::fs::create_dir_all(&workspace_path).unwrap();

        let config = temp_env::with_vars(
            [
                ("HOME", temp_dir().to_str()),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), None, None, None, false, false).unwrap(),
        );

        assert_eq!(config.threads, *LOGICAL_CPUS)
    }

    #[test]
    fn test_toml_has_precedence_when_multiple_configs_present() {
        let workspace_path = temp_dir().join("workspace-with-multiple-configs");
        std::fs::create_dir_all(&workspace_path).unwrap();

        create_tmp_file("threads = 3", &workspace_path, "toml");
        create_tmp_file("threads: 2\nphp-version: \"7.4.0\"", &workspace_path, "yaml");
        create_tmp_file("{\"threads\": 1,\"php-version\":\"8.1.0\"}", &workspace_path, "json");

        let config = Configuration::load(Some(workspace_path), None, None, None, false, false).unwrap();

        assert_eq!(config.threads, 3);
        assert_eq!(config.php_version.to_string(), DEFAULT_PHP_VERSION.to_string())
    }

    #[test]
    fn test_env_config_override_all_others() {
        let workspace_path = temp_dir().join("workspace-1");
        let config_path = temp_dir().join("config-1");

        std::fs::create_dir_all(&workspace_path).unwrap();
        std::fs::create_dir_all(&config_path).unwrap();

        let config_file_path = create_tmp_file("threads = 1", &config_path, "toml");
        create_tmp_file("threads = 2", &workspace_path, "toml");

        let config = temp_env::with_vars(
            [
                ("HOME", None),
                ("MAGO_THREADS", Some("3")),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), Some(&config_file_path), None, None, false, false).unwrap(),
        );

        assert_eq!(config.threads, 3);
    }

    #[test]
    fn test_config_cancel_workspace() {
        let workspace_path = temp_dir().join("workspace-2");
        let config_path = temp_dir().join("config-2");

        std::fs::create_dir_all(&workspace_path).unwrap();
        std::fs::create_dir_all(&config_path).unwrap();

        create_tmp_file("threads = 2\nphp-version = \"7.4.0\"", &workspace_path, "toml");

        let config_file_path = create_tmp_file("threads = 1", &config_path, "toml");
        let config = temp_env::with_vars(
            [
                ("HOME", None::<&str>),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), Some(&config_file_path), None, None, false, false).unwrap(),
        );

        assert_eq!(config.threads, 1);
        assert_eq!(config.php_version.to_string(), DEFAULT_PHP_VERSION.to_string());
    }

    #[test]
    fn test_workspace_has_precedence_over_global() {
        let home_path = temp_dir().join("home-3");
        let xdg_config_home_path = temp_dir().join("xdg-config-home-3");
        let workspace_path = temp_dir().join("workspace-3");

        let _ = std::fs::remove_dir_all(&home_path);
        let _ = std::fs::remove_dir_all(&xdg_config_home_path);
        let _ = std::fs::remove_dir_all(&workspace_path);

        std::fs::create_dir_all(&home_path).unwrap();
        std::fs::create_dir_all(&xdg_config_home_path).unwrap();
        std::fs::create_dir_all(&workspace_path).unwrap();

        create_tmp_file("threads: 2\nphp-version: \"8.1.0\"", &workspace_path.to_owned(), "yaml");
        create_tmp_file("threads = 3\nphp-version = \"7.4.0\"", &home_path, "toml");
        create_tmp_file("source.excludes = [\"yes\"]", &xdg_config_home_path, "toml");

        let config = temp_env::with_vars(
            [
                ("HOME", Some(home_path)),
                ("XDG_CONFIG_HOME", Some(xdg_config_home_path)),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path.clone()), None, None, None, false, false).unwrap(),
        );

        assert_eq!(config.threads, 2);
        assert_eq!(config.php_version.to_string(), "8.1.0".to_string());
        assert_eq!(config.source.excludes, Vec::<String>::new());
    }

    fn create_tmp_file(config_content: &str, folder: &PathBuf, extension: &str) -> PathBuf {
        fs::create_dir_all(folder).unwrap();
        let config_path = folder.join(CONFIGURATION_FILE_NAME).with_extension(extension);
        fs::write(&config_path, config_content).unwrap();
        config_path
    }

    fn write_file(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn load_isolated(file: &Path) -> Configuration {
        temp_env::with_vars(
            [
                ("HOME", None::<&str>),
                ("XDG_CONFIG_HOME", None),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(None, Some(file), None, None, false, false).unwrap(),
        )
    }

    #[test]
    fn test_extends_single_string() {
        let dir = temp_dir().join("extends-single");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        write_file(&dir.join("base.toml"), "threads = 7\nphp-version = \"8.0.0\"\n");
        write_file(&dir.join("mago.toml"), "extends = \"base.toml\"\nphp-version = \"8.3.0\"\n");

        let config = load_isolated(&dir.join("mago.toml"));
        assert_eq!(config.threads, 7);
        assert_eq!(config.php_version.to_string(), "8.3.0");
    }

    #[test]
    fn test_extends_array_in_order() {
        let dir = temp_dir().join("extends-array");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        write_file(&dir.join("a.toml"), "threads = 1\nphp-version = \"8.0.0\"\n");
        write_file(&dir.join("b.toml"), "threads = 2\n");
        write_file(&dir.join("mago.toml"), "extends = [\"a.toml\", \"b.toml\"]\n");

        let config = load_isolated(&dir.join("mago.toml"));
        assert_eq!(config.threads, 2);
        assert_eq!(config.php_version.to_string(), "8.0.0");
    }

    #[test]
    fn test_extends_relative_to_config_file_not_cwd() {
        let dir = temp_dir().join("extends-relative");
        let _ = fs::remove_dir_all(&dir);
        let nested = dir.join("nested");
        fs::create_dir_all(&nested).unwrap();

        write_file(&nested.join("base.toml"), "threads = 9\n");
        write_file(&nested.join("mago.toml"), "extends = \"base.toml\"\n");

        let config = load_isolated(&nested.join("mago.toml"));
        assert_eq!(config.threads, 9);
    }

    #[test]
    fn test_extends_directory_picks_up_mago_file() {
        let dir = temp_dir().join("extends-dir");
        let _ = fs::remove_dir_all(&dir);
        let configs = dir.join("configs");
        fs::create_dir_all(&configs).unwrap();

        write_file(&configs.join("mago.toml"), "threads = 5\n");
        write_file(&dir.join("mago.toml"), "extends = \"configs\"\n");

        let config = load_isolated(&dir.join("mago.toml"));
        assert_eq!(config.threads, 5);
    }

    #[test]
    fn test_extends_directory_without_config_warns_and_skips() {
        let dir = temp_dir().join("extends-empty-dir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("empty")).unwrap();
        write_file(&dir.join("mago.toml"), "extends = \"empty\"\nthreads = 3\n");

        let config = load_isolated(&dir.join("mago.toml"));
        assert_eq!(config.threads, 3);
    }

    #[test]
    fn test_extends_array_excludes_concat() {
        let dir = temp_dir().join("extends-array-concat");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        write_file(&dir.join("base.toml"), "[source]\nexcludes = [\"vendor\", \"node_modules\"]\n");
        write_file(&dir.join("mago.toml"), "extends = \"base.toml\"\n[source]\nexcludes = [\"build\"]\n");

        let config = load_isolated(&dir.join("mago.toml"));
        assert_eq!(config.source.excludes, vec!["vendor", "node_modules", "build"]);
    }

    #[test]
    fn test_extends_cycle_is_detected() {
        let dir = temp_dir().join("extends-cycle");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        write_file(&dir.join("a.toml"), "extends = \"b.toml\"\n");
        write_file(&dir.join("b.toml"), "extends = \"a.toml\"\n");

        let result = Configuration::load(None, Some(&dir.join("a.toml")), None, None, false, false);
        assert!(result.is_err(), "expected cycle to be detected");
    }

    #[test]
    fn test_extends_transitive_chain() {
        let dir = temp_dir().join("extends-transitive");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        write_file(&dir.join("grandparent.toml"), "threads = 1\nphp-version = \"8.0.0\"\n");
        write_file(&dir.join("parent.toml"), "extends = \"grandparent.toml\"\nthreads = 2\n");
        write_file(&dir.join("mago.toml"), "extends = \"parent.toml\"\n");

        let config = load_isolated(&dir.join("mago.toml"));
        assert_eq!(config.threads, 2);
        assert_eq!(config.php_version.to_string(), "8.0.0");
    }

    #[test]
    fn test_extends_mixed_formats() {
        let dir = temp_dir().join("extends-mixed");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        write_file(&dir.join("base.json"), "{\"threads\": 4}\n");
        write_file(&dir.join("middle.yaml"), "extends: \"base.json\"\nphp-version: \"8.2.0\"\n");
        write_file(&dir.join("mago.toml"), "extends = \"middle.yaml\"\n");

        let config = load_isolated(&dir.join("mago.toml"));
        assert_eq!(config.threads, 4);
        assert_eq!(config.php_version.to_string(), "8.2.0");
    }
}

/// Auto-detect the editor URL template from environment hints.
///
/// Checks terminal environment variables to determine which editor is running,
/// and returns the appropriate URL template for clickable file paths.
fn detect_editor_url() -> Option<String> {
    if let Ok(bundle_id) = std::env::var("__CFBundleIdentifier") {
        let url = match bundle_id.as_str() {
            "com.jetbrains.PhpStorm" | "com.jetbrains.PhpStorm-EAP" => {
                "phpstorm://open?file=%file%&line=%line%&column=%column%"
            }
            "com.jetbrains.intellij" | "com.jetbrains.intellij.ce" => {
                "idea://open?file=%file%&line=%line%&column=%column%"
            }
            "com.jetbrains.WebStorm" | "com.jetbrains.WebStorm-EAP" => {
                "webstorm://open?file=%file%&line=%line%&column=%column%"
            }
            "dev.zed.Zed" | "dev.zed.Zed-Preview" => "zed://file/%file%:%line%:%column%",
            "com.microsoft.VSCode" => "vscode://file/%file%:%line%:%column%",
            "com.microsoft.VSCodeInsiders" => "vscode-insiders://file/%file%:%line%:%column%",
            "com.sublimetext.4" | "com.sublimetext.3" => "subl://open?url=file://%file%&line=%line%&column=%column%",
            _ => return None,
        };

        tracing::debug!("Auto-detected editor URL from __CFBundleIdentifier={bundle_id}");

        return Some(url.to_string());
    }

    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        let url = match term_program.as_str() {
            "vscode" => "vscode://file/%file%:%line%:%column%",
            "zed" => "zed://file/%file%:%line%:%column%",
            _ => return None,
        };

        tracing::debug!("Auto-detected editor URL from TERM_PROGRAM={term_program}");

        return Some(url.to_string());
    }

    None
}

/// Configuration file format. Order of variants is the precedence order used during
/// auto-discovery within a directory: TOML wins over YAML wins over JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

impl ConfigFormat {
    pub(crate) const ALL: &'static [ConfigFormat] = &[ConfigFormat::Toml, ConfigFormat::Yaml, ConfigFormat::Json];

    /// Extensions matched for this format, in preference order.
    pub(crate) fn extensions(&self) -> &'static [&'static str] {
        match self {
            ConfigFormat::Toml => &["toml"],
            ConfigFormat::Yaml => &["yaml", "yml"],
            ConfigFormat::Json => &["json"],
        }
    }

    /// Detect format from a file path's extension; returns `None` if unrecognised.
    pub(crate) fn for_path(path: &Path) -> Option<ConfigFormat> {
        let ext = path.extension().and_then(|e| e.to_str())?;
        for f in Self::ALL {
            if f.extensions().iter().any(|x| x.eq_ignore_ascii_case(ext)) {
                return Some(*f);
            }
        }
        None
    }

    /// Parse raw file content into a generic `serde_json::Value` tree using the right serde
    /// driver. We use `serde_json::Value` as the universal merge type because it covers TOML,
    /// YAML, and JSON (TOML datetimes get coerced to strings, which is fine — Configuration
    /// doesn't have datetime fields). `path` is included in error messages.
    pub(crate) fn parse_to_value(&self, content: &str, path: &Path) -> Result<Value, Error> {
        match self {
            ConfigFormat::Toml => toml::from_str::<Value>(content)
                .map_err(|e| Error::ParseConfigFile { path: path.to_path_buf(), source: Box::new(e) }),
            ConfigFormat::Yaml => serde_yml::from_str::<Value>(content)
                .map_err(|e| Error::ParseConfigFile { path: path.to_path_buf(), source: Box::new(e) }),
            ConfigFormat::Json => serde_json::from_str::<Value>(content)
                .map_err(|e| Error::ParseConfigFile { path: path.to_path_buf(), source: Box::new(e) }),
        }
    }
}

/// Recursively load a config file into its merged `serde_json::Value` representation.
///
/// Each layer's `extends` directive is processed before the layer's own values are applied,
/// so the precedence order is: defaults < extends[0] < extends[1] < … < this file's keys.
/// `extends` paths are resolved relative to the directory of the file declaring them (not
/// against cwd) — important when running with `--config some/dir/config.toml`.
fn load_layer(path: &Path, format: ConfigFormat, visited: &mut HashSet<PathBuf>) -> Result<Value, Error> {
    let canonical = path.canonicalize().map_err(|e| Error::ReadConfigFile { path: path.to_path_buf(), source: e })?;
    if visited.contains(&canonical) {
        return Err(Error::CircularExtends(canonical));
    }
    visited.insert(canonical);

    let content =
        std::fs::read_to_string(path).map_err(|e| Error::ReadConfigFile { path: path.to_path_buf(), source: e })?;
    let mut value = format.parse_to_value(&content, path)?;

    let extends = extract_extends(&mut value, path)?;
    if extends.is_empty() {
        return Ok(value);
    }

    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut accumulator = Value::Object(serde_json::Map::new());
    for entry in &extends {
        match resolve_extends_entry(entry, base_dir)? {
            Some((resolved_path, resolved_format)) => {
                tracing::debug!("Extending configuration from {}.", resolved_path.display());
                let parent_value = load_layer(&resolved_path, resolved_format, visited)?;
                merge_into(&mut accumulator, parent_value);
            }
            None => {
                tracing::warn!(
                    "Configuration `extends` entry `{}` (resolved relative to `{}`) is a directory \
                     without a `mago.toml`/`mago.yaml`/`mago.yml`/`mago.json` — skipping.",
                    entry,
                    base_dir.display()
                );
            }
        }
    }

    merge_into(&mut accumulator, value);

    Ok(accumulator)
}

/// Pop the `extends` key off the top-level table and normalise it to a list of strings.
/// Errors if it's present but not a string or array of strings.
fn extract_extends(value: &mut Value, path: &Path) -> Result<Vec<String>, Error> {
    let Some(obj) = value.as_object_mut() else {
        return Ok(Vec::new());
    };
    let Some(raw) = obj.remove("extends") else {
        return Ok(Vec::new());
    };

    match raw {
        Value::String(s) => Ok(vec![s]),
        Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for v in arr {
                match v {
                    Value::String(s) => out.push(s),
                    other => {
                        return Err(Error::InvalidExtendsEntry {
                            path: path.to_path_buf(),
                            reason: format!("expected string, got {}", json_value_kind(&other)),
                        });
                    }
                }
            }
            Ok(out)
        }
        other => Err(Error::InvalidExtendsEntry {
            path: path.to_path_buf(),
            reason: format!("expected string or array of strings, got {}", json_value_kind(&other)),
        }),
    }
}

/// Resolve a single `extends` entry against `base_dir` (the directory of the file declaring
/// the extends). Returns `None` if the entry is a directory with no recognised config file
/// inside (caller logs a warning and skips). Returns an error if the entry doesn't exist or
/// has an unrecognised extension.
fn resolve_extends_entry(entry: &str, base_dir: &Path) -> Result<Option<(PathBuf, ConfigFormat)>, Error> {
    let entry_path = Path::new(entry);
    let resolved = if entry_path.is_absolute() { entry_path.to_path_buf() } else { base_dir.join(entry_path) };

    let metadata = std::fs::metadata(&resolved).map_err(|e| Error::ExtendsTargetNotFound {
        entry: entry.to_string(),
        resolved: resolved.clone(),
        source: e,
    })?;

    if metadata.is_dir() {
        // Reuse a single PathBuf; only its trailing extension changes per probe.
        let mut candidate = resolved.join(CONFIGURATION_FILE_NAME);
        for format in ConfigFormat::ALL {
            for ext in format.extensions() {
                candidate.set_extension(ext);
                if candidate.exists() {
                    return Ok(Some((candidate, *format)));
                }
            }
        }

        return Ok(None);
    }

    let format =
        ConfigFormat::for_path(&resolved).ok_or_else(|| Error::UnsupportedConfigExtension(resolved.clone()))?;
    Ok(Some((resolved, format)))
}

/// Recursively merge `source` into `target`. Object keys from `source` override / merge with
/// `target`'s. Arrays are concatenated (target first, source second). Scalars in `source`
/// replace scalars in `target`.
fn merge_into(target: &mut Value, source: Value) {
    use serde_json::Value;
    match (target, source) {
        (Value::Object(t), Value::Object(s)) => {
            for (k, v) in s {
                match t.get_mut(&k) {
                    Some(existing) => merge_into(existing, v),
                    None => {
                        t.insert(k, v);
                    }
                }
            }
        }
        (Value::Array(t), Value::Array(s)) => {
            t.extend(s);
        }
        (target, source) => {
            *target = source;
        }
    }
}

fn json_value_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Parse a boolean from an env var. Accepts: 1/0, true/false, yes/no, on/off (any case).
fn parse_bool(s: &str) -> Result<bool, std::io::Error> {
    let trimmed = s.trim();
    if trimmed == "1"
        || trimmed.eq_ignore_ascii_case("true")
        || trimmed.eq_ignore_ascii_case("yes")
        || trimmed.eq_ignore_ascii_case("on")
    {
        return Ok(true);
    }

    if trimmed.is_empty()
        || trimmed == "0"
        || trimmed.eq_ignore_ascii_case("false")
        || trimmed.eq_ignore_ascii_case("no")
        || trimmed.eq_ignore_ascii_case("off")
    {
        return Ok(false);
    }

    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid boolean: `{s}`")))
}
