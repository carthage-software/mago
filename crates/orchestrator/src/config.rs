//! Configuration types for the orchestrator.
//!
//! This module defines [`OrchestratorConfiguration`], which aggregates all settings
//! needed by the orchestrator and its various services.

use std::path::Path;

use mago_analyzer::settings::Settings as AnalyzerSettings;
use mago_formatter::settings::FormatSettings;
use mago_guard::settings::Settings as GuardSettings;
use mago_linter::settings::Settings as LinterSettings;
use mago_php_version::PHPVersion;

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

    /// Directories containing source files to analyze.
    ///
    /// These are the primary targets for linting, formatting, and analysis. If empty,
    /// the entire workspace directory will be scanned for PHP files.
    ///
    /// # Examples
    ///
    /// - `vec![PathBuf::from("src")]` - Only analyze files in the `src` directory
    /// - `vec![PathBuf::from("src"), PathBuf::from("tests")]` - Analyze both `src` and `tests`
    /// - `vec![]` - Scan the entire workspace
    pub paths: Vec<&'a Path>,

    /// Additional files or directories to include for context.
    ///
    /// Files in this list provide context for analysis (e.g., vendor dependencies) but
    /// are not directly analyzed, linted, or formatted themselves. This is useful for
    /// including third-party code that provides type information without actually checking
    /// that code.
    pub includes: Vec<&'a Path>,

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
