//! Configuration types for the orchestrator.
//!
//! This module defines [`OrchestratorConfiguration`], which aggregates all settings
//! needed by the orchestrator and its various services.

use mago_analyzer::settings::Settings as AnalyzerSettings;
use mago_formatter::settings::FormatSettings;
use mago_guard::settings::Settings as GuardSettings;
use mago_linter::settings::Settings as LinterSettings;
use mago_php_version::PHPVersion;
use mago_syntax::settings::ParserSettings;

/// The complete configuration for the orchestrator and all its services.
///
/// This struct acts as a "meta-configuration" that aggregates all settings needed by the
/// various analysis tools (linter, formatter, analyzer, guard) along with global settings
/// that apply across all tools.
///
/// # Structure
///
/// The configuration is organized into three main categories:
///
/// 1. **Global Settings**: PHP version, progress bars, colors
/// 2. **File Discovery**: Paths, includes, excludes, extensions
/// 3. **Tool Settings**: Linter, analyzer, guard, formatter configurations
#[derive(Debug)]
pub struct OrchestratorConfiguration<'a> {
    /// The PHP version to use for parsing and analysis.
    ///
    /// This determines which language features are recognized and how code is parsed.
    /// For example, `PHPVersion::PHP82` enables PHP 8.2 features like readonly classes
    /// and disjunctive normal form types.
    pub php_version: PHPVersion,

    /// Paths or glob patterns for source files to analyze.
    ///
    /// These are the primary targets for linting, formatting, and analysis. If empty,
    /// the entire workspace directory will be scanned for PHP files.
    ///
    /// Supports both directory paths and glob patterns:
    /// - Directory paths: `"src"`, `"tests"` - recursively scans all files
    /// - Glob patterns: `"src/**/*.php"`, `"tests/Unit/*Test.php"` - matches specific files
    ///
    /// # Examples
    ///
    /// - `vec!["src"]` - Only analyze files in the `src` directory
    /// - `vec!["src", "tests"]` - Analyze both `src` and `tests`
    /// - `vec!["src/**/*.php"]` - Only analyze PHP files in src using glob pattern
    /// - `vec![]` - Scan the entire workspace
    pub paths: Vec<String>,

    /// Paths or glob patterns for files to include for context.
    ///
    /// Files in this list provide context for analysis (e.g., vendor dependencies) but
    /// are not directly analyzed, linted, or formatted themselves. This is useful for
    /// including third-party code that provides type information without actually checking
    /// that code.
    ///
    /// Supports both directory paths and glob patterns (same as `paths`).
    pub includes: Vec<String>,

    /// Glob patterns or paths to exclude from file scanning.
    ///
    /// These patterns are used to filter out files and directories that should not be
    /// processed by any tool. Patterns can use glob syntax with wildcards.
    ///
    /// # Examples
    ///
    /// - `"*.tmp"` - Exclude all temporary files
    /// - `"build/*"` - Exclude everything in the build directory
    /// - `"vendor/**"` - Exclude all vendor directories recursively
    /// - `"./cache"` - Exclude a specific directory relative to the workspace root
    pub excludes: Vec<&'a str>,

    /// File extensions to treat as PHP files.
    ///
    /// Only files with these extensions will be processed. The default is typically
    /// just `["php"]`, but you can add others like `"phtml"`, `"php8"`, etc.
    pub extensions: Vec<&'a str>,

    /// Settings for the parser.
    ///
    /// Controls lexer and parser behavior, such as whether short open tags (`<?`) are enabled.
    /// See [`mago_syntax::settings::ParserSettings`] for available options.
    pub parser_settings: ParserSettings,

    /// Settings for the static analyzer.
    ///
    /// Controls type checking, control flow analysis, and other deep analysis features.
    /// See [`mago_analyzer::settings::Settings`] for available options.
    pub analyzer_settings: AnalyzerSettings,

    /// Settings for the linter.
    ///
    /// Controls which linting rules are enabled and their configuration.
    /// See [`mago_linter::settings::Settings`] for available options.
    pub linter_settings: LinterSettings,

    /// Settings for the architectural guard.
    ///
    /// Defines architectural layers and their allowed dependencies.
    /// See [`mago_guard::settings::Settings`] for available options.
    pub guard_settings: GuardSettings,

    /// Settings for the code formatter.
    ///
    /// Controls code style preferences like indentation, line length, etc.
    /// See [`mago_formatter::settings::FormatSettings`] for available options.
    pub formatter_settings: FormatSettings,

    /// Disable all default analyzer plugins (including stdlib).
    ///
    /// When set to `true`, no plugins will be loaded by default, and only plugins
    /// explicitly listed in `analyzer_plugins` will be enabled.
    pub disable_default_analyzer_plugins: bool,

    /// List of analyzer plugins to enable (by name or alias).
    ///
    /// Plugins can be specified by their canonical name or any of their aliases:
    /// - `stdlib` (aliases: `standard`, `std`, `php-stdlib`)
    /// - `psl` (aliases: `php-standard-library`, `azjezz-psl`)
    /// - `flow-php` (aliases: `flow`, `flow-etl`)
    /// - `psr-container` (aliases: `psr-11`)
    pub analyzer_plugins: Vec<String>,

    /// Whether to display progress bars during long-running operations.
    ///
    /// Progress bars provide visual feedback in terminal environments but should be
    /// disabled in CI/CD pipelines or when piping output to files.
    ///
    /// **Default**: `false` (for library users)
    pub use_progress_bars: bool,

    /// Whether to use colors in output.
    ///
    /// Color output improves readability in terminals but should be disabled when
    /// piping to files or in environments that don't support ANSI color codes.
    ///
    /// **Default**: `false` (for library users)
    pub use_colors: bool,
}
