//! Interactive configuration initialization command.
//!
//! This module implements the `mago init` command, which provides an interactive guided setup
//! for creating a new `mago.toml` configuration file. The initialization process walks users
//! through configuring all aspects of Mago including source paths, linter rules, formatter
//! settings, and analyzer options.
//!
//! # Setup Process
//!
//! The initialization is divided into five steps:
//!
//! 1. **Project Setup**: Configure PHP version, source paths, includes, and excludes
//! 2. **Linter Configuration**: Select framework integrations and rules
//! 3. **Formatter Configuration**: Set code style preferences
//! 4. **Analyzer Configuration**: Enable analysis features and categories
//! 5. **Review & Confirm**: Preview and write the configuration file
//!
//! # Auto-Detection
//!
//! The command intelligently detects project settings from `composer.json` when available:
//!
//! - **PHP Version**: Extracted from the `require.php` constraint
//! - **Source Paths**: Derived from PSR-4 autoload configuration
//! - **Framework Integrations**: Detected from required packages (Laravel, Symfony, etc.)
//!
//! # Configuration Template
//!
//! The generated configuration file is based on a TOML template with placeholder values
//! that are replaced based on user selections. The template includes sensible defaults
//! and common customizations for PER-CS compliance.

use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use clap::Parser;
use colored::Colorize;
use dialoguer::Confirm;
use dialoguer::Input;
use dialoguer::MultiSelect;
use dialoguer::Select;
use dialoguer::console::style;
use dialoguer::theme::ColorfulTheme;

use mago_composer::AutoloadPsr4value;
use mago_composer::ComposerPackage;
use mago_composer::ComposerPackageAutoloadDevPsr4value;
use mago_formatter::presets::FormatterPreset;
use mago_linter::integration::Integration;
use mago_php_version::PHPVersion;

use crate::config::Configuration;
use crate::consts::COMPOSER_JSON_FILE;
use crate::consts::CONFIGURATION_FILE_NAME;
use crate::consts::DEFAULT_PHP_VERSION;
use crate::error::Error;
use crate::utils::version::extract_minimum_php_version;

/// Available analyzer plugins that can be enabled during initialization.
///
/// These plugins provide specialized type inference for specific libraries,
/// improving the analyzer's understanding of library-specific functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalyzerPlugin {
    /// Type providers for azjezz/psl package
    Psl,
    /// Type providers for flow-php/etl package
    FlowPhp,
}

impl std::fmt::Display for AnalyzerPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Psl => write!(f, "psl"),
            Self::FlowPhp => write!(f, "flow-php"),
        }
    }
}

impl AnalyzerPlugin {
    /// Returns a human-readable description of the plugin.
    fn description(&self) -> &'static str {
        match self {
            Self::Psl => "PSL - Type providers for azjezz/psl package",
            Self::FlowPhp => "Flow-PHP - Type providers for flow-php/etl package",
        }
    }
}

/// Strictness presets for the analyzer configuration.
///
/// These presets provide pre-configured sets of analyzer settings ranging from
/// minimal checks (suitable for legacy codebases) to maximum strictness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AnalyzerStrictnessPreset {
    /// Minimal checks, good for legacy codebases or gradual adoption
    Relaxed,
    /// Sensible defaults for most projects (default)
    #[default]
    Balanced,
    /// Enable most checks for clean, well-maintained codebases
    Strict,
    /// All checks enabled, strictest possible settings
    Maximum,
}

impl std::fmt::Display for AnalyzerStrictnessPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Relaxed => write!(f, "Relaxed - Minimal checks, good for legacy codebases"),
            Self::Balanced => write!(f, "Balanced - Sensible defaults for most projects"),
            Self::Strict => write!(f, "Strict - Enable most checks for clean codebases"),
            Self::Maximum => write!(f, "Maximum - All checks enabled, strictest settings"),
        }
    }
}

impl AnalyzerStrictnessPreset {
    /// Converts this preset to concrete analyzer settings.
    fn to_settings(self, plugins: Vec<AnalyzerPlugin>) -> InitializationAnalyzerSettings {
        match self {
            Self::Relaxed => InitializationAnalyzerSettings {
                plugins,
                find_unused_definitions: false,
                find_unused_expressions: false,
                analyze_dead_code: false,
                check_throws: false,
                allow_possibly_undefined_array_keys: true,
                check_missing_override: false,
                find_unused_parameters: false,
                memoize_properties: true,
                strict_list_index_checks: false,
                no_boolean_literal_comparison: false,
                check_missing_type_hints: false,
                register_super_globals: true,
            },
            Self::Balanced => InitializationAnalyzerSettings {
                plugins,
                find_unused_definitions: true,
                find_unused_expressions: false,
                analyze_dead_code: false,
                check_throws: false,
                allow_possibly_undefined_array_keys: true,
                check_missing_override: false,
                find_unused_parameters: false,
                memoize_properties: true,
                strict_list_index_checks: false,
                no_boolean_literal_comparison: false,
                check_missing_type_hints: false,
                register_super_globals: true,
            },
            Self::Strict => InitializationAnalyzerSettings {
                plugins,
                find_unused_definitions: true,
                find_unused_expressions: true,
                analyze_dead_code: false,
                check_throws: true,
                allow_possibly_undefined_array_keys: false,
                check_missing_override: true,
                find_unused_parameters: true,
                memoize_properties: true,
                strict_list_index_checks: true,
                no_boolean_literal_comparison: false,
                check_missing_type_hints: false,
                register_super_globals: true,
            },
            Self::Maximum => InitializationAnalyzerSettings {
                plugins,
                find_unused_definitions: true,
                find_unused_expressions: true,
                analyze_dead_code: true,
                check_throws: true,
                allow_possibly_undefined_array_keys: false,
                check_missing_override: true,
                find_unused_parameters: true,
                memoize_properties: true,
                strict_list_index_checks: true,
                no_boolean_literal_comparison: true,
                check_missing_type_hints: true,
                register_super_globals: true,
            },
        }
    }
}

/// TOML template for the generated `mago.toml` configuration file.
///
/// This template uses placeholder markers (e.g., `{php_version}`, `{paths}`) that are
/// replaced with user-selected values during the initialization process. The template
/// provides a well-structured configuration with all major sections:
///
/// - PHP version targeting
/// - Source path configuration
/// - Formatter style settings (PER-CS compatible by default)
/// - Linter rules and integrations
/// - Analyzer features and options
const CONFIGURATION_TEMPLATE: &str = r#"# Welcome to Mago!
# For full documentation, see https://mago.carthage.software/tools/overview
php-version = "{php_version}"

[source]
workspace = "."
paths = [{paths}]
includes = [{includes}]
excludes = [{excludes}]

{formatter_config}

[linter]
integrations = [{integrations}]

[linter.rules]
ambiguous-function-call = { enabled = false }
literal-named-argument = { enabled = false }
halstead = { effort-threshold = 7000 }

[analyzer]
plugins = [{analyzer_plugins}]
{analyzer_settings}
"#;

/// Command for initializing a new Mago configuration file.
///
/// This command provides an interactive guided setup process for creating a `mago.toml`
/// configuration file. It walks users through five steps to configure all aspects of Mago,
/// with intelligent auto-detection from `composer.json` when available.
///
/// # Behavior
///
/// - If `mago.toml` exists, offers to back it up before proceeding
/// - Attempts to auto-detect settings from `composer.json` if present
/// - Provides interactive prompts for all configuration options
/// - Generates a complete, ready-to-use `mago.toml` file
///
/// # User Experience
///
/// The command uses a colorful,  well-formatted terminal interface with:
/// - Step-by-step progress indicators
/// - Helpful descriptions for each option
/// - Sensible defaults for all settings
/// - Preview of the final configuration before writing
#[derive(Parser, Debug)]
#[command(
    name = "init",
    about = "Initialize Mago for your project with a guided setup.",
    long_about = "Creates a new mago.toml configuration file by walking you through a setup process."
)]
pub struct InitCommand {}

impl InitCommand {
    /// Executes the interactive initialization process.
    ///
    /// This method orchestrates the complete initialization workflow:
    ///
    /// 1. Displays a welcome banner
    /// 2. Checks for existing configuration and offers backup if needed
    /// 3. Guides the user through five setup steps:
    ///    - Project settings (paths, PHP version)
    ///    - Linter configuration (integrations, rules)
    ///    - Formatter settings (style preferences)
    ///    - Analyzer configuration (features, issue categories)
    ///    - Review and confirmation
    /// 4. Writes the final configuration to `mago.toml`
    ///
    /// # Arguments
    ///
    /// * `configuration` - The base configuration (used for workspace path)
    /// * `configuration_file` - Optional path override for the config file location
    ///
    /// # Returns
    ///
    /// Always returns `Ok(ExitCode::SUCCESS)` since initialization can be cancelled
    /// at any point without being considered a failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Reading `composer.json` fails when user opts to use it
    /// - Writing the configuration file fails due to permissions or I/O errors
    /// - Terminal interaction fails (non-interactive environment)
    pub fn execute(self, configuration: Configuration, configuration_file: Option<PathBuf>) -> Result<ExitCode, Error> {
        let theme = ColorfulTheme {
            prompt_prefix: style("".to_string()),
            success_prefix: style("".to_string()),
            error_prefix: style("".to_string()),
            ..Default::default()
        };

        let configuration_file = configuration_file
            .unwrap_or_else(|| configuration.source.workspace.join(CONFIGURATION_FILE_NAME).with_extension("toml"));

        print_welcome_banner();
        if configuration_file.exists() {
            println!("  ⚠️  {}", "Mago is already configured!".bold().yellow());
            println!("      {}", format!("Found mago.toml at: {}", configuration_file.display()).bright_black());
            println!();

            if Confirm::with_theme(&theme)
                .with_prompt("    Do you want to back up the existing file and start over?")
                .default(false)
                .interact()?
            {
                let backup_path = configuration_file.with_extension("toml.bkp");
                std::fs::rename(&configuration_file, &backup_path).map_err(Error::WritingConfiguration)?;

                println!();
                println!("  ✅ {}", "Backed up existing configuration.".bold().green());
                println!("      {}", format!("Moved to: {}", backup_path.display()).bright_black());
            } else {
                println!();
                println!("  ❌ {}", "Initialization cancelled.".yellow());
                println!();

                return Ok(ExitCode::SUCCESS);
            }
        }

        let InitializationProjectSettings { php_version, paths, includes, excludes } = setup_project(&theme)?;

        let integrations = setup_linter(&theme)?;
        let formatter_config = setup_formatter(&theme)?;
        let analyzer_settings = setup_analyzer(&theme)?;

        print_step_header(5, "Review & Confirm");
        let config_content = CONFIGURATION_TEMPLATE
            .replace("{php_version}", &php_version)
            .replace("{paths}", &quote_format_strings(&paths))
            .replace("{includes}", &quote_format_strings(&includes))
            .replace("{excludes}", &quote_format_strings(&excludes))
            .replace(
                "{integrations}",
                &quote_format_strings(&integrations.iter().map(|i| i.to_string().to_lowercase()).collect::<Vec<_>>()),
            )
            .replace("{formatter_config}", &formatter_config)
            .replace(
                "{analyzer_plugins}",
                &quote_format_strings(&analyzer_settings.plugins.iter().map(|p| p.to_string()).collect::<Vec<_>>()),
            )
            .replace("{analyzer_settings}", &build_analyzer_settings_string(&analyzer_settings));

        if write_configuration_if_confirmed(&theme, &configuration_file, &config_content)? {
            print_final_summary();
        } else {
            println!("  ❌ {}", "Initialization cancelled. No file was written.".yellow());
            println!();
        }

        Ok(ExitCode::SUCCESS)
    }
}

/// Project configuration settings collected during initialization.
///
/// This struct holds the basic project setup gathered from either auto-detection
/// (via `composer.json`) or user prompts, including the target PHP version and
/// file path configurations.
#[derive(Debug)]
struct InitializationProjectSettings {
    /// Target PHP version (e.g., "8.2", "8.3")
    php_version: String,
    /// Source code paths to analyze (e.g., ["src", "tests"])
    paths: Vec<String>,
    /// Dependency paths for context (e.g., ["vendor"])
    includes: Vec<String>,
    /// Paths to exclude from all processing (e.g., ["build", "dist"])
    excludes: Vec<String>,
}

/// Analyzer configuration settings collected during initialization.
///
/// This struct holds all analyzer-specific settings gathered from user prompts,
/// including feature toggles. These settings determine the depth and scope of
/// static analysis.
#[derive(Debug)]
struct InitializationAnalyzerSettings {
    /// Analyzer plugins to enable
    plugins: Vec<AnalyzerPlugin>,
    /// Whether to find unused definitions (classes, functions, methods)
    find_unused_definitions: bool,
    /// Whether to find unused expressions (statements with no effect)
    find_unused_expressions: bool,
    /// Whether to analyze code paths that appear unreachable
    analyze_dead_code: bool,
    /// Whether to check for unhandled thrown exceptions
    check_throws: bool,
    /// Whether to allow access to potentially undefined array keys
    allow_possibly_undefined_array_keys: bool,
    /// Whether to check for missing #[Override] attributes
    check_missing_override: bool,
    /// Whether to find unused function/method parameters
    find_unused_parameters: bool,
    /// Whether to track literal values of class properties
    memoize_properties: bool,
    /// Whether to enforce strict checks when accessing list elements by index
    strict_list_index_checks: bool,
    /// Whether to disallow comparisons with boolean literals
    no_boolean_literal_comparison: bool,
    /// Whether to check for missing type hints
    check_missing_type_hints: bool,
    /// Whether to register superglobals in analysis context
    register_super_globals: bool,
}

fn print_welcome_banner() {
    println!();
    println!("{}", " Mago".bold().cyan());
    println!();
    println!("{}", " ⬩ Welcome! Let's get you set up.".bright_black());
    println!();
}

fn print_step_header(step: u8, title: &str) {
    println!("  ╭─ {} {}", format!("Step {}:", step).bold(), title.cyan().bold());
    println!("  │");
}

fn print_final_summary() {
    println!();
    println!("  ╭─ 🎉 {}", "You're all set!".bold().cyan());
    println!("  │");
    println!("  │  {}", "Mago is now configured for your project.".bold());
    println!("  │");
    println!("  │  {}", "What's next?".underline());
    println!("  │    - Run {} to check for issues.", "`mago lint`".yellow());
    println!("  │    - Run {} to find type errors.", "`mago analyze`".yellow());
    println!("  │    - See formatting changes with {}.", "`mago fmt --dry-run`".yellow());
    println!("  │");
    println!("  │  {}", "Tip: Use the `--help` flag on any command for more options.".bright_black());
    println!("  │");
    println!("  ╰─ {}", "For full documentation, visit: https://mago.carthage.software/".underline());
    println!();
}

fn setup_project(theme: &ColorfulTheme) -> Result<InitializationProjectSettings, Error> {
    print_step_header(1, "Project Setup");
    let composer_file = Path::new(COMPOSER_JSON_FILE);
    if composer_file.exists()
        && Confirm::with_theme(theme)
            .with_prompt(format!(
                " │  Found `{}`. Use it to auto-configure project paths & PHP version?",
                COMPOSER_JSON_FILE
            ))
            .default(true)
            .interact()?
    {
        println!("  │");
        println!("  │  {}", "Reading composer.json...".bright_black());
        let composer_json = std::fs::read_to_string(composer_file).map_err(Error::ReadingComposerJson)?;
        let composer = ComposerPackage::from_str(&composer_json)?;
        let workspace = composer_file.parent().unwrap_or_else(|| Path::new("."));

        let php_version = extract_php_version_from_composer(&composer);
        let paths = extract_paths_from_composer(&composer, workspace);
        let includes = vec!["vendor".to_string()];
        let excludes = Vec::new();

        println!("  │  {}", "Project settings detected!".green());
        println!("  ╰─");

        Ok(InitializationProjectSettings { php_version, paths, includes, excludes })
    } else {
        println!("  │");
        let paths = prompt_for_paths(theme, "Source code paths (e.g., src,tests)", Some("src,tests"))?;
        let includes = prompt_for_paths(theme, "Dependency paths (e.g., vendor)", Some("vendor"))?;
        let excludes = prompt_for_paths(theme, "Paths to exclude (e.g., build,dist)", None)?;
        let php_version = prompt_for_php_version(theme)?;
        println!("  ╰─");

        Ok(InitializationProjectSettings { php_version, paths, includes, excludes })
    }
}

fn setup_linter(theme: &ColorfulTheme) -> Result<Vec<Integration>, Error> {
    print_step_header(2, "Linter Configuration");
    println!("  │  {}", "The Linter checks your code for stylistic issues and inconsistencies.".bright_black());
    println!("  │  {}", "It helps keep your codebase clean and readable.".bright_black());
    println!("  │");

    let composer_file = Path::new(COMPOSER_JSON_FILE);
    if composer_file.exists()
        && Confirm::with_theme(theme)
            .with_prompt(" │  Use `composer.json` to auto-detect framework integrations?")
            .default(true)
            .interact()?
    {
        println!("  │");
        println!("  │  {}", "Detecting integrations from composer.json...".bright_black());
        let composer_json = std::fs::read_to_string(composer_file).map_err(Error::ReadingComposerJson)?;
        let composer = ComposerPackage::from_str(&composer_json)?;
        let integrations = detect_integrations_from_composer(&composer);
        println!("  │  {}", "Done!".green());
        println!("  ╰─");
        Ok(integrations)
    } else {
        let integrations = prompt_for_integrations(theme)?;
        println!("  ╰─");
        Ok(integrations)
    }
}

fn setup_formatter(theme: &ColorfulTheme) -> Result<String, Error> {
    print_step_header(3, "Formatter Configuration");
    println!("  │  {}", "The Formatter automatically rewrites your files to a consistent style.".bright_black());
    println!("  │  {}", "This ends debates about spacing and helps you focus on the code.".bright_black());
    println!("  │");

    if Confirm::with_theme(theme)
        .with_prompt(" │  Do you want to use a preset formatter configuration?")
        .default(false)
        .interact()?
    {
        println!("  │");

        let preset_values = FormatterPreset::all();
        let preset_items = preset_values.iter().map(|p| p.description()).collect::<Vec<_>>();
        let selection =
            Select::with_theme(theme).with_prompt(" │  Select a preset").items(&preset_items).default(0).interact()?;

        let selected_preset = preset_values[selection];

        println!("  │");
        println!("  │  {}", format!("Selected preset: {}", preset_items[selection]).green());
        println!("  ╰─");

        Ok(format!("[formatter]\npreset = \"{}\"", selected_preset))
    } else {
        let defaults = (120, 4, false);

        if Confirm::with_theme(theme)
            .with_prompt(" │  The default settings are PER-CS compatible. Do you want to customize them?")
            .default(false)
            .interact()?
        {
            println!("  │");
            let print_width = prompt_for_u16(theme, " │  Print width (line length)", defaults.0)?;
            let tab_width = prompt_for_u8(theme, " │  Tab width (spaces)", defaults.1)?;
            let use_tabs = Confirm::with_theme(theme)
                .with_prompt(" │  Use tabs instead of spaces?")
                .default(defaults.2)
                .interact()?;
            println!("  │");
            println!(
                "  │  {}",
                "ℹ️  The formatter has many more options. Check the docs to customize it further.".blue()
            );
            println!("  ╰─");
            Ok(format!(
                "[formatter]\nprint-width = {}\ntab-width = {}\nuse-tabs = {}",
                print_width, tab_width, use_tabs
            ))
        } else {
            println!("  │");
            println!("  │  {}", "Great choice! Sticking to the defaults is highly recommended.".green());
            println!("  ╰─");
            Ok(format!(
                "[formatter]\nprint-width = {}\ntab-width = {}\nuse-tabs = {}",
                defaults.0, defaults.1, defaults.2
            ))
        }
    }
}

fn setup_analyzer(theme: &ColorfulTheme) -> Result<InitializationAnalyzerSettings, Error> {
    print_step_header(4, "Analyzer Configuration");
    println!("  │  {}", "The Analyzer finds logical bugs and type errors before you run your code.".bright_black());
    println!("  │  {}", "This is the most powerful part of Mago.".bright_black());
    println!("  │");

    // Step 1: Plugin detection/selection
    let plugins = setup_analyzer_plugins(theme)?;

    // Step 2: Strictness preset selection
    println!("  │");
    println!("  │  {}", "Choose a strictness level for the analyzer:".bright_black());
    println!("  │");

    let presets = &[
        AnalyzerStrictnessPreset::Relaxed,
        AnalyzerStrictnessPreset::Balanced,
        AnalyzerStrictnessPreset::Strict,
        AnalyzerStrictnessPreset::Maximum,
    ];

    let selection = Select::with_theme(theme)
        .with_prompt(" │  Strictness preset")
        .items(presets)
        .default(1) // Balanced
        .interact()?;

    let preset = presets[selection];
    let mut settings = preset.to_settings(plugins);

    // Step 3: Optional customization
    println!("  │");
    if Confirm::with_theme(theme)
        .with_prompt(" │  Would you like to customize individual settings?")
        .default(false)
        .interact()?
    {
        println!("  │");
        println!("  │  {}", "Detection Settings:".underline());
        settings.find_unused_definitions = Confirm::with_theme(theme)
            .with_prompt(" │  Find unused definitions (e.g., private methods)?")
            .default(settings.find_unused_definitions)
            .interact()?;
        settings.find_unused_expressions = Confirm::with_theme(theme)
            .with_prompt(" │  Find unused expressions (e.g., `$a + $b;`)?")
            .default(settings.find_unused_expressions)
            .interact()?;
        settings.analyze_dead_code = Confirm::with_theme(theme)
            .with_prompt(" │  Analyze code that appears to be unreachable?")
            .default(settings.analyze_dead_code)
            .interact()?;

        println!("  │");
        println!("  │  {}", "Type Checking:".underline());
        settings.check_throws = Confirm::with_theme(theme)
            .with_prompt(" │  Check for unhandled thrown exceptions?")
            .default(settings.check_throws)
            .interact()?;
        settings.check_missing_type_hints = Confirm::with_theme(theme)
            .with_prompt(" │  Check for missing type hints?")
            .default(settings.check_missing_type_hints)
            .interact()?;
        settings.no_boolean_literal_comparison = Confirm::with_theme(theme)
            .with_prompt(" │  Disallow comparisons with boolean literals?")
            .default(settings.no_boolean_literal_comparison)
            .interact()?;

        println!("  │");
        println!("  │  {}", "Array Handling:".underline());
        settings.allow_possibly_undefined_array_keys = Confirm::with_theme(theme)
            .with_prompt(" │  Allow accessing possibly undefined array keys?")
            .default(settings.allow_possibly_undefined_array_keys)
            .interact()?;
        settings.strict_list_index_checks = Confirm::with_theme(theme)
            .with_prompt(" │  Enforce strict checks for list index access?")
            .default(settings.strict_list_index_checks)
            .interact()?;

        println!("  │");
        println!("  │  {}", "Other Settings:".underline());
        settings.check_missing_override = Confirm::with_theme(theme)
            .with_prompt(" │  Check for missing #[Override] attributes (PHP 8.3+)?")
            .default(settings.check_missing_override)
            .interact()?;
        settings.find_unused_parameters = Confirm::with_theme(theme)
            .with_prompt(" │  Find unused function/method parameters?")
            .default(settings.find_unused_parameters)
            .interact()?;
        settings.memoize_properties = Confirm::with_theme(theme)
            .with_prompt(" │  Track literal values of class properties?")
            .default(settings.memoize_properties)
            .interact()?;
        settings.register_super_globals = Confirm::with_theme(theme)
            .with_prompt(" │  Register superglobals ($_GET, $_POST, etc.)?")
            .default(settings.register_super_globals)
            .interact()?;
    }

    println!("  ╰─");
    Ok(settings)
}

fn setup_analyzer_plugins(theme: &ColorfulTheme) -> Result<Vec<AnalyzerPlugin>, Error> {
    let composer_file = Path::new(COMPOSER_JSON_FILE);

    if composer_file.exists()
        && Confirm::with_theme(theme)
            .with_prompt(" │  Use `composer.json` to auto-detect analyzer plugins?")
            .default(true)
            .interact()?
    {
        println!("  │");
        println!("  │  {}", "Detecting plugins from composer.json...".bright_black());
        let composer_json = std::fs::read_to_string(composer_file).map_err(Error::ReadingComposerJson)?;
        let composer = ComposerPackage::from_str(&composer_json)?;
        let plugins = detect_analyzer_plugins_from_composer(&composer);

        if plugins.is_empty() {
            println!("  │  {}", "No plugins detected.".bright_black());
        } else {
            for plugin in &plugins {
                println!("  │  {} {}", "✓".green(), plugin.description());
            }
            println!("  │  {}", "Done!".green());
        }

        Ok(plugins)
    } else {
        println!("  │");
        prompt_for_analyzer_plugins(theme)
    }
}

fn write_configuration_if_confirmed(
    theme: &ColorfulTheme,
    config_path: &Path,
    config_content: &str,
) -> Result<bool, Error> {
    println!("  │");
    println!("  │  {}", "Your `mago.toml` file will look like this:".bright_black());
    println!("  │");
    println!("  │  {}", "```toml".bright_black());
    for line in config_content.trim().lines() {
        println!("  │  {}", line.green());
    }
    println!("  │  {}", "```".bright_black());
    println!("  │");

    if Confirm::with_theme(theme).with_prompt(" │  Write configuration to `mago.toml`?").default(true).interact()? {
        std::fs::write(config_path, config_content.trim()).map_err(Error::WritingConfiguration)?;
        println!("  ╰─");
        println!();
        println!("  ✅ {}", "Configuration file created successfully!".bold().green());
        Ok(true)
    } else {
        println!("  ╰─");
        Ok(false)
    }
}

fn extract_php_version_from_composer(composer: &ComposerPackage) -> String {
    composer
        .require
        .get("php")
        .and_then(|constraint| extract_minimum_php_version(constraint))
        .unwrap_or_else(|| DEFAULT_PHP_VERSION.to_string())
}

fn extract_paths_from_composer(composer: &ComposerPackage, workspace: &Path) -> Vec<String> {
    let mut paths = Vec::new();

    if let Some(autoload) = composer.autoload.as_ref() {
        paths.extend(autoload.psr_4.values().flat_map(get_autoload_value));
    }
    if let Some(autoload_dev) = composer.autoload_dev.as_ref() {
        paths.extend(autoload_dev.psr_4.values().flat_map(get_autoload_dev_value));
    }

    let existing_paths: Vec<String> = paths.into_iter().filter(|p| workspace.join(p).exists()).collect();
    deduplicate_paths(existing_paths)
}

fn deduplicate_paths(mut paths: Vec<String>) -> Vec<String> {
    if paths.len() <= 1 {
        return paths;
    }
    paths.sort();
    paths.dedup();

    let mut parent_paths = Vec::new();
    for path in &paths {
        if !parent_paths.iter().any(|p: &String| path.starts_with(&format!("{}/", p.trim_end_matches('/')))) {
            parent_paths.push(path.clone());
        }
    }
    parent_paths
}

fn detect_integrations_from_composer(composer: &ComposerPackage) -> Vec<Integration> {
    let mut integrations = vec![];
    if has_package(composer, "azjezz/psl") {
        integrations.push(Integration::Psl);
    }

    if has_package_prefix(composer, "symfony/") {
        integrations.push(Integration::Symfony);
    }

    if has_package_prefix(composer, "laravel/") {
        integrations.push(Integration::Laravel);
    }

    if has_package(composer, "phpunit/phpunit") {
        integrations.push(Integration::PHPUnit);
    }

    if has_package_prefix(composer, "pestphp/") {
        integrations.push(Integration::Pest);
    }

    if has_package(composer, "tempest/framework") {
        integrations.push(Integration::Tempest);
    }

    integrations
}

fn detect_analyzer_plugins_from_composer(composer: &ComposerPackage) -> Vec<AnalyzerPlugin> {
    let mut plugins = vec![];

    if has_package(composer, "azjezz/psl") {
        plugins.push(AnalyzerPlugin::Psl);
    }

    if has_package_prefix(composer, "flow-php/") {
        plugins.push(AnalyzerPlugin::FlowPhp);
    }

    plugins
}

fn has_package_prefix(composer: &ComposerPackage, prefix: &str) -> bool {
    composer.require.keys().any(|k| k.starts_with(prefix)) || composer.require_dev.keys().any(|k| k.starts_with(prefix))
}

fn has_package(composer: &ComposerPackage, package_name: &str) -> bool {
    composer.require.contains_key(package_name) || composer.require_dev.contains_key(package_name)
}

fn get_autoload_value(autoload: &AutoloadPsr4value) -> Vec<String> {
    match autoload {
        AutoloadPsr4value::Array(items) => items.clone(),
        AutoloadPsr4value::String(path) => vec![path.clone()],
    }
}

fn get_autoload_dev_value(autoload: &ComposerPackageAutoloadDevPsr4value) -> Vec<String> {
    match autoload {
        ComposerPackageAutoloadDevPsr4value::Array(items) => items.clone(),
        ComposerPackageAutoloadDevPsr4value::String(path) => vec![path.clone()],
    }
}

fn prompt_for_paths(theme: &ColorfulTheme, prompt: &str, default: Option<&str>) -> Result<Vec<String>, Error> {
    let mut builder = Input::with_theme(theme);
    builder = builder.with_prompt(format!(" │  {}", prompt)).allow_empty(true);

    if let Some(d) = default {
        builder = builder.default(d.to_string());
    }

    let input: String = builder.interact_text()?;

    if input.is_empty() {
        return Ok(default.unwrap_or("").split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect());
    }

    Ok(input.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
}

fn prompt_for_php_version(theme: &ColorfulTheme) -> Result<String, Error> {
    let input: String = Input::with_theme(theme)
        .with_prompt(" │  PHP version to target")
        .default(DEFAULT_PHP_VERSION.to_string())
        .allow_empty(true)
        .validate_with(|v: &String| {
            if v.is_empty() {
                return Ok(());
            }
            PHPVersion::from_str(v).map(|_| ()).map_err(|e| e.to_string())
        })
        .interact_text()?;

    Ok(if input.is_empty() { DEFAULT_PHP_VERSION.to_string() } else { input })
}

fn prompt_for_u16(theme: &ColorfulTheme, prompt: &str, default: u16) -> Result<u16, Error> {
    Input::with_theme(theme).with_prompt(prompt).default(default).interact_text().map_err(Error::from)
}

fn prompt_for_u8(theme: &ColorfulTheme, prompt: &str, default: u8) -> Result<u8, Error> {
    Input::with_theme(theme).with_prompt(prompt).default(default).interact_text().map_err(Error::from)
}

fn prompt_for_integrations(theme: &ColorfulTheme) -> Result<Vec<Integration>, Error> {
    let items = &[
        Integration::Psl,
        Integration::Laravel,
        Integration::Pest,
        Integration::Tempest,
        Integration::PHPUnit,
        Integration::Symfony,
    ];

    let selections = MultiSelect::with_theme(theme)
        .with_prompt(" │  Select integrations to enable (space to select, enter to confirm)")
        .items(items)
        .interact()?;

    Ok(selections.into_iter().map(|i| items[i]).collect())
}

fn prompt_for_analyzer_plugins(theme: &ColorfulTheme) -> Result<Vec<AnalyzerPlugin>, Error> {
    let items = &[AnalyzerPlugin::Psl, AnalyzerPlugin::FlowPhp];

    let descriptions: Vec<&str> = items.iter().map(|p| p.description()).collect();

    let selections = MultiSelect::with_theme(theme)
        .with_prompt(" │  Select analyzer plugins (space to select, enter to confirm)")
        .items(&descriptions)
        .interact()?;

    Ok(selections.into_iter().map(|i| items[i]).collect())
}

fn quote_format_strings(items: &[String]) -> String {
    items.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", ")
}

fn build_analyzer_settings_string(settings: &InitializationAnalyzerSettings) -> String {
    let mut lines = Vec::new();

    lines.push(format!("find-unused-definitions = {}", settings.find_unused_definitions));
    lines.push(format!("find-unused-expressions = {}", settings.find_unused_expressions));
    lines.push(format!("analyze-dead-code = {}", settings.analyze_dead_code));
    lines.push(format!("memoize-properties = {}", settings.memoize_properties));
    lines.push(format!("allow-possibly-undefined-array-keys = {}", settings.allow_possibly_undefined_array_keys));
    lines.push(format!("check-throws = {}", settings.check_throws));
    lines.push(format!("check-missing-override = {}", settings.check_missing_override));
    lines.push(format!("find-unused-parameters = {}", settings.find_unused_parameters));
    lines.push(format!("strict-list-index-checks = {}", settings.strict_list_index_checks));
    lines.push(format!("no-boolean-literal-comparison = {}", settings.no_boolean_literal_comparison));
    lines.push(format!("check-missing-type-hints = {}", settings.check_missing_type_hints));
    lines.push(format!("register-super-globals = {}", settings.register_super_globals));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::Configuration;

    fn create_default_analyzer_settings() -> InitializationAnalyzerSettings {
        InitializationAnalyzerSettings {
            plugins: vec![],
            find_unused_definitions: true,
            find_unused_expressions: false,
            analyze_dead_code: false,
            check_throws: false,
            allow_possibly_undefined_array_keys: true,
            check_missing_override: false,
            find_unused_parameters: false,
            memoize_properties: true,
            strict_list_index_checks: false,
            no_boolean_literal_comparison: false,
            check_missing_type_hints: false,
            register_super_globals: true,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn generate_config_content(
        php_version: &str,
        paths: &[String],
        includes: &[String],
        excludes: &[String],
        integrations: &[Integration],
        formatter_config: &str,
        analyzer_settings: &InitializationAnalyzerSettings,
    ) -> String {
        CONFIGURATION_TEMPLATE
            .replace("{php_version}", php_version)
            .replace("{paths}", &quote_format_strings(paths))
            .replace("{includes}", &quote_format_strings(includes))
            .replace("{excludes}", &quote_format_strings(excludes))
            .replace(
                "{integrations}",
                &quote_format_strings(&integrations.iter().map(|i| i.to_string().to_lowercase()).collect::<Vec<_>>()),
            )
            .replace("{formatter_config}", formatter_config)
            .replace(
                "{analyzer_plugins}",
                &quote_format_strings(&analyzer_settings.plugins.iter().map(|p| p.to_string()).collect::<Vec<_>>()),
            )
            .replace("{analyzer_settings}", &build_analyzer_settings_string(analyzer_settings))
    }

    #[test]
    fn test_generated_config_parses_with_defaults() {
        let formatter_config = format!("[formatter]\nprint-width = {}\ntab-width = {}\nuse-tabs = {}", 120, 4, false);
        let content = generate_config_content(
            "8.2",
            &["src".to_string()],
            &["vendor".to_string()],
            &[],
            &[],
            &formatter_config,
            &create_default_analyzer_settings(),
        );

        let result: Result<Configuration, _> = toml::from_str(&content);
        assert!(result.is_ok(), "Generated config should parse. Error: {:?}\n\nConfig:\n{}", result.err(), content);
    }

    #[test]
    fn test_generated_config_parses_with_all_options() {
        let settings = InitializationAnalyzerSettings {
            plugins: vec![AnalyzerPlugin::Psl, AnalyzerPlugin::FlowPhp],
            find_unused_definitions: true,
            find_unused_expressions: true,
            analyze_dead_code: true,
            check_throws: true,
            allow_possibly_undefined_array_keys: false,
            check_missing_override: false,
            find_unused_parameters: false,
            memoize_properties: true,
            strict_list_index_checks: true,
            no_boolean_literal_comparison: true,
            check_missing_type_hints: true,
            register_super_globals: false,
        };
        let formatter_config = format!("[formatter]\nprint-width = {}\ntab-width = {}\nuse-tabs = {}", 100, 2, true);

        let content = generate_config_content(
            "8.4",
            &["src".to_string(), "app".to_string()],
            &["vendor".to_string()],
            &["tests".to_string()],
            &[Integration::Symfony, Integration::PHPUnit],
            &formatter_config,
            &settings,
        );

        let result: Result<Configuration, _> = toml::from_str(&content);
        assert!(result.is_ok(), "Generated config should parse. Error: {:?}\n\nConfig:\n{}", result.err(), content);
    }

    #[test]
    fn test_generated_config_parses_with_integrations() {
        let formatter_config = format!("[formatter]\nprint-width = {}\ntab-width = {}\nuse-tabs = {}", 120, 4, false);
        let content = generate_config_content(
            "8.3",
            &["src".to_string()],
            &["vendor".to_string()],
            &[],
            &[Integration::Psl, Integration::Laravel, Integration::PHPUnit, Integration::Symfony],
            &formatter_config,
            &create_default_analyzer_settings(),
        );

        let result: Result<Configuration, _> = toml::from_str(&content);
        assert!(result.is_ok(), "Generated config should parse. Error: {:?}\n\nConfig:\n{}", result.err(), content);
    }

    #[test]
    fn test_analyzer_settings_string_generation() {
        let settings = create_default_analyzer_settings();
        let output = build_analyzer_settings_string(&settings);

        assert!(output.contains("find-unused-definitions = true"));
        assert!(output.contains("find-unused-expressions = false"));
        assert!(output.contains("analyze-dead-code = false"));
        assert!(output.contains("memoize-properties = true"));
        assert!(output.contains("allow-possibly-undefined-array-keys = true"));
        assert!(output.contains("check-throws = false"));
        assert!(output.contains("check-missing-override = false"));
        assert!(output.contains("find-unused-parameters = false"));
        assert!(output.contains("strict-list-index-checks = false"));
        assert!(output.contains("no-boolean-literal-comparison = false"));
        assert!(output.contains("check-missing-type-hints = false"));
        assert!(output.contains("register-super-globals = true"));
    }

    #[test]
    fn test_analyzer_strictness_presets() {
        // Test Relaxed preset
        let relaxed = AnalyzerStrictnessPreset::Relaxed.to_settings(vec![]);
        assert!(!relaxed.find_unused_definitions);
        assert!(!relaxed.find_unused_expressions);
        assert!(!relaxed.analyze_dead_code);
        assert!(!relaxed.check_throws);
        assert!(relaxed.allow_possibly_undefined_array_keys);

        // Test Balanced preset
        let balanced = AnalyzerStrictnessPreset::Balanced.to_settings(vec![]);
        assert!(balanced.find_unused_definitions);
        assert!(!balanced.find_unused_expressions);
        assert!(!balanced.check_throws);
        assert!(balanced.allow_possibly_undefined_array_keys);

        // Test Strict preset
        let strict = AnalyzerStrictnessPreset::Strict.to_settings(vec![]);
        assert!(strict.find_unused_definitions);
        assert!(strict.find_unused_expressions);
        assert!(strict.check_throws);
        assert!(!strict.allow_possibly_undefined_array_keys);
        assert!(strict.check_missing_override);
        assert!(strict.find_unused_parameters);

        // Test Maximum preset
        let maximum = AnalyzerStrictnessPreset::Maximum.to_settings(vec![]);
        assert!(maximum.find_unused_definitions);
        assert!(maximum.find_unused_expressions);
        assert!(maximum.analyze_dead_code);
        assert!(maximum.check_throws);
        assert!(!maximum.allow_possibly_undefined_array_keys);
        assert!(maximum.check_missing_type_hints);
        assert!(maximum.no_boolean_literal_comparison);
    }

    #[test]
    fn test_analyzer_plugin_display() {
        assert_eq!(AnalyzerPlugin::Psl.to_string(), "psl");
        assert_eq!(AnalyzerPlugin::FlowPhp.to_string(), "flow-php");
    }

    #[test]
    fn test_generated_config_with_analyzer_plugins() {
        let settings =
            InitializationAnalyzerSettings { plugins: vec![AnalyzerPlugin::Psl], ..create_default_analyzer_settings() };
        let formatter_config = "[formatter]\nprint-width = 120\ntab-width = 4\nuse-tabs = false";

        let content = generate_config_content(
            "8.3",
            &["src".to_string()],
            &["vendor".to_string()],
            &[],
            &[],
            formatter_config,
            &settings,
        );

        assert!(content.contains(r#"plugins = ["psl"]"#));

        let result: Result<Configuration, _> = toml::from_str(&content);
        assert!(result.is_ok(), "Generated config should parse. Error: {:?}\n\nConfig:\n{}", result.err(), content);
    }
}
