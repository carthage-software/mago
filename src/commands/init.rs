use std::path::Path;
use std::process::ExitCode;
use std::str::FromStr;

use clap::Parser;
use dialoguer::Confirm;
use dialoguer::Input;
use dialoguer::MultiSelect;
use dialoguer::theme::ColorfulTheme;

use mago_composer::AutoloadPsr4value;
use mago_composer::ComposerPackage;
use mago_composer::ComposerPackageAutoloadDevPsr4value;
use mago_php_version::PHPVersion;

use crate::config::Configuration;
use crate::consts::COMPOSER_JSON_FILE;
use crate::consts::CONFIGURATION_FILE;
use crate::consts::DEFAULT_PHP_VERSION;
use crate::error::Error;
use crate::utils::version::extract_minimum_php_version;

/// Template for generating the mago.toml configuration file.
const CONFIGURATION_TEMPLATE: &str = r#"
# Mago configuration file
# For more information, see https://mago.carthage.software/#/getting-started/configuration
php_version = "{php_version}"

[source]
paths = [{paths}]
includes = [{includes}]
excludes = [{excludes}]

[format]
print_width = 120
tab_width = 4
use_tabs = false

[linter]
default_plugins = true
plugins = [{plugins}]
"#;

const SYMFONY_PLUGIN: &str = "symfony";
const LARAVEL_PLUGIN: &str = "laravel";
const PHPUNIT_PLUGIN: &str = "php-unit";

/// Available plugin options for PHP frameworks and libraries.
const PLUGIN_OPTIONS: [&str; 3] = [LARAVEL_PLUGIN, SYMFONY_PLUGIN, PHPUNIT_PLUGIN];

#[derive(Parser, Debug)]
#[command(
    name = "init",
    about = "Initialize the configuration for Mago",
    long_about = r#"
Initialize a new Mago configuration file (mago.toml) in the current workspace.

This command guides you through the process of setting up Mago for your PHP project.
It can automatically detect and use settings from an existing composer.json file,
or help you configure settings manually including:

- PHP version to target
- Source code paths to analyze
- External dependency paths
- Paths to exclude from analysis
- Framework-specific plugins to enable (Laravel, Symfony, PHPUnit)

The generated configuration will be written to mago.toml in the current workspace.
"#
)]
pub struct InitCommand;

/// Executes the `init` command, returning an exit status code.
///
/// This function coordinates the overall configuration creation process:
/// 1. Checks if a configuration file already exists
/// 2. Detects if composer.json is available and should be used
/// 3. Gathers configuration data (either from composer.json or user input)
/// 4. Generates and writes the configuration file
///
/// # Arguments
///
/// * `_command` - The init command instance with any parameters
/// * `configuration` - The current configuration context
///
/// # Returns
///
/// A result containing the exit code or an error
pub async fn execute(_command: InitCommand, configuration: Configuration) -> Result<ExitCode, Error> {
    let theme = ColorfulTheme::default();

    // Check if configuration file already exists
    let configuration_file = configuration.source.workspace.join(CONFIGURATION_FILE);
    if configuration_file.exists() {
        tracing::info!("Configuration file already exists at {}", configuration_file.display());

        return Ok(ExitCode::SUCCESS);
    }

    // Find composer.json and decide whether to use it
    let composer_file = configuration.source.workspace.join(COMPOSER_JSON_FILE);
    let use_composer = composer_file.exists() && should_use_composer(&composer_file)?;

    // Generate configuration content
    let configuration = if use_composer {
        generate_config_from_composer(&composer_file)?
    } else {
        generate_config_from_user_input(&theme)?
    };

    // Confirm and write the configuration file
    write_configuration_if_confirmed(&theme, &configuration_file, &configuration)?;

    Ok(ExitCode::SUCCESS)
}

/// Asks the user if they want to use the detected composer.json file for configuration.
///
/// # Arguments
///
/// * `composer_file` - Path to the composer.json file
///
/// # Returns
///
/// A result containing a boolean indicating whether to use composer.json or an error
fn should_use_composer(composer_file: &Path) -> Result<bool, Error> {
    let theme = ColorfulTheme::default();

    tracing::info!("Found `composer.json` file at {}", composer_file.display());

    let should_use_composer =
        Confirm::with_theme(&theme).with_prompt("Use `composer.json` to configure Mago?").default(true).interact()?;

    Ok(should_use_composer)
}

/// Generates configuration content based on composer.json file.
///
/// # Arguments
///
/// * `composer_file` - Path to the composer.json file
///
/// # Returns
///
/// A result containing the configuration content as a string or an error
fn generate_config_from_composer(composer_file: &Path) -> Result<String, Error> {
    // Parse composer.json
    let composer_json = std::fs::read_to_string(composer_file).map_err(Error::ReadingComposerJson)?;
    let composer = ComposerPackage::from_str(&composer_json).map_err(Error::ParsingComposerJson)?;

    // Extract PHP version
    let php_version = extract_php_version_from_composer(&composer);

    // Extract paths from autoload configuration
    let paths = extract_paths_from_composer(&composer);

    // Standard include path for Composer projects
    let includes = vec!["vendor".to_string()];

    // Detect and enable appropriate plugins based on dependencies
    let plugins = detect_plugins_from_composer(&composer);

    // Generate configuration content
    Ok(CONFIGURATION_TEMPLATE
        .replace("{php_version}", &php_version)
        .replace("{paths}", &quote_format_strings(paths))
        .replace("{includes}", &quote_format_strings(includes))
        .replace("{excludes}", "")
        .replace("{plugins}", &quote_format_strings(plugins)))
}

/// Extracts the PHP version from composer package requirements.
///
/// # Arguments
///
/// * `composer` - Parsed composer.json content
///
/// # Returns
///
/// The PHP version as a string
fn extract_php_version_from_composer(composer: &ComposerPackage) -> String {
    match composer.require.get("php") {
        Some(version_constraint) => match extract_minimum_php_version(version_constraint) {
            Some(version) => version,
            None => DEFAULT_PHP_VERSION.to_string(),
        },
        None => DEFAULT_PHP_VERSION.to_string(),
    }
}

/// Extracts source code paths from composer autoload configuration.
///
/// # Arguments
///
/// * `composer` - Parsed composer.json content
///
/// # Returns
///
/// A vector of path strings
fn extract_paths_from_composer(composer: &ComposerPackage) -> Vec<String> {
    let mut paths = match composer.autoload.as_ref() {
        Some(autoload) => autoload.psr_4.iter().flat_map(|(_, v)| get_autoload_value(v)).collect::<Vec<_>>(),
        None => vec![],
    };

    if let Some(autoload) = composer.autoload_dev.as_ref() {
        paths.extend(autoload.psr_4.iter().flat_map(|(_, v)| get_autoload_dev_value(v)));
    }

    paths
}

/// Detects which plugins should be enabled based on composer dependencies.
///
/// # Arguments
///
/// * `composer` - Parsed composer.json content
///
/// # Returns
///
/// A vector of plugin names to enable
fn detect_plugins_from_composer(composer: &ComposerPackage) -> Vec<String> {
    let mut plugins = vec![];

    // Detect Symfony framework
    if has_package_prefix(composer, "symfony/") || has_package_prefix(composer, "symfony-") {
        plugins.push(SYMFONY_PLUGIN.to_string());
    }

    // Detect Laravel framework
    if has_package_prefix(composer, "laravel/") || has_package_prefix(composer, "illuminate/") {
        plugins.push(LARAVEL_PLUGIN.to_string());
    }

    // Detect PHPUnit
    if has_exact_package(composer, "phpunit/phpunit") || has_exact_package(composer, "symfony/phpunit-bridge") {
        plugins.push(PHPUNIT_PLUGIN.to_string());
    }

    plugins
}

/// Checks if the composer dependencies include a package with the given prefix.
///
/// # Arguments
///
/// * `composer` - Parsed composer.json content
/// * `prefix` - Package name prefix to check for
///
/// # Returns
///
/// Boolean indicating if a matching package was found
fn has_package_prefix(composer: &ComposerPackage, prefix: &str) -> bool {
    composer.require.iter().any(|(k, _)| k.starts_with(prefix))
        || composer.require_dev.iter().any(|(k, _)| k.starts_with(prefix))
}

/// Checks if the composer dependencies include an exact package name.
///
/// # Arguments
///
/// * `composer` - Parsed composer.json content
/// * `package_name` - Exact package name to check for
///
/// # Returns
///
/// Boolean indicating if the package was found
fn has_exact_package(composer: &ComposerPackage, package_name: &str) -> bool {
    composer.require.iter().any(|(k, _)| k.eq(package_name))
        || composer.require_dev.iter().any(|(k, _)| k.eq(package_name))
}

/// Extracts PSR-4 autoload paths from composer configuration.
///
/// # Arguments
///
/// * `autoload` - Autoload value from composer.json
///
/// # Returns
///
/// A vector of path strings
fn get_autoload_value(autoload: &AutoloadPsr4value) -> Vec<String> {
    match autoload {
        AutoloadPsr4value::Array(items) => items.iter().map(|p| p.to_string()).collect(),
        AutoloadPsr4value::String(path) => vec![path.to_string()],
    }
}

/// Extracts PSR-4 autoload-dev paths from composer configuration.
///
/// # Arguments
///
/// * `autoload` - Autoload-dev value from composer.json
///
/// # Returns
///
/// A vector of path strings
fn get_autoload_dev_value(autoload: &ComposerPackageAutoloadDevPsr4value) -> Vec<String> {
    match autoload {
        ComposerPackageAutoloadDevPsr4value::Array(items) => items.iter().map(|p| p.to_string()).collect(),
        ComposerPackageAutoloadDevPsr4value::String(path) => vec![path.to_string()],
    }
}

/// Generates configuration content based on user input.
///
/// # Arguments
///
/// * `theme` - Dialoguer theme for input prompts
///
/// # Returns
///
/// A result containing the configuration content as a string or an error
fn generate_config_from_user_input(theme: &ColorfulTheme) -> Result<String, Error> {
    // Collect source paths
    let paths = prompt_for_paths(theme, "Paths to include for analysis (comma-separated, e.g., src,tests)")?;

    // Collect include paths for external dependencies
    let includes =
        prompt_for_paths(theme, "Paths to include for external dependencies (comma-separated, e.g., vendor)")?;

    // Collect exclude paths
    let excludes = prompt_for_paths(theme, "Paths to exclude from analysis (comma-separated, e.g., src/Generated)")?;

    // Prompt for PHP version
    let php_version = prompt_for_php_version(theme)?;

    // Select plugins
    let plugins = prompt_for_plugins(theme)?;

    // Generate configuration content
    Ok(CONFIGURATION_TEMPLATE
        .replace("{php_version}", &php_version)
        .replace("{paths}", &quote_format_strings(paths))
        .replace("{includes}", &quote_format_strings(includes))
        .replace("{excludes}", &quote_format_strings(excludes))
        .replace("{plugins}", &quote_format_strings(plugins)))
}

/// Prompts the user for a comma-separated list of paths.
///
/// # Arguments
///
/// * `theme` - Dialoguer theme for input prompts
/// * `prompt` - Prompt message to display
///
/// # Returns
///
/// A result containing a vector of paths or an error
fn prompt_for_paths(theme: &ColorfulTheme, prompt: &str) -> Result<Vec<String>, Error> {
    let input = Input::<String>::with_theme(theme)
        .with_prompt(prompt)
        .validate_with(|v: &String| paths_validator(v))
        .allow_empty(true)
        .interact()?;

    Ok(input.split(',').map(|e| e.trim().to_string()).filter(|e| !e.is_empty()).collect())
}

/// Prompts the user for the PHP version to target.
///
/// # Arguments
///
/// * `theme` - Dialoguer theme for input prompts
///
/// # Returns
///
/// A result containing the PHP version as a string or an error
fn prompt_for_php_version(theme: &ColorfulTheme) -> Result<String, Error> {
    let php_version: String = Input::<String>::with_theme(theme)
        .with_prompt("PHP version to target (e.g., 8.3, 8.4)")
        .allow_empty(true)
        .validate_with(|v: &String| {
            if v.is_empty() {
                return Ok(());
            }

            match PHPVersion::from_str(v) {
                Ok(_) => Ok(()),
                Err(error) => Err(error.to_string()),
            }
        })
        .interact()?;

    Ok(if php_version.is_empty() { DEFAULT_PHP_VERSION.to_string() } else { php_version })
}

/// Prompts the user to select which plugins to enable.
///
/// # Arguments
///
/// * `theme` - Dialoguer theme for input prompts
///
/// # Returns
///
/// A result containing a vector of selected plugin names or an error
fn prompt_for_plugins(theme: &ColorfulTheme) -> Result<Vec<String>, Error> {
    let selected_indices = MultiSelect::with_theme(theme)
        .with_prompt("Select framework and library plugins to enable")
        .items(&PLUGIN_OPTIONS)
        .interact()?;

    Ok(selected_indices.iter().map(|&idx| PLUGIN_OPTIONS[idx].to_string()).collect())
}

/// Validates path input to ensure it's properly formatted.
///
/// # Arguments
///
/// * `v` - Input string containing comma-separated paths
///
/// # Returns
///
/// Ok if valid, Err with message if invalid
fn paths_validator(v: &str) -> Result<(), &'static str> {
    if v.is_empty() {
        return Ok(());
    }

    if v.contains(|c: char| c.is_whitespace()) {
        return Err("Paths cannot contain whitespaces. Use commas to separate multiple paths.");
    }

    let paths = v.split(',').map(|p| p.trim()).collect::<Vec<_>>();

    if paths.iter().all(|p| !p.is_empty()) {
        return Ok(());
    }

    Err("Paths cannot be empty.")
}

/// Confirms with the user and writes the configuration file.
///
/// # Arguments
///
/// * `theme` - Dialoguer theme for confirmation prompt
/// * `configuration_file` - Path where the configuration should be written
/// * `configuration` - Configuration content to write
///
/// # Returns
///
/// A result with () or an error
fn write_configuration_if_confirmed(
    theme: &ColorfulTheme,
    configuration_file: &Path,
    configuration: &str,
) -> Result<(), Error> {
    let should_write =
        Confirm::with_theme(theme).with_prompt("Write configuration to `mago.toml`?").default(true).interact()?;

    if should_write {
        tracing::info!("Writing configuration to {}", configuration_file.display());
        std::fs::write(configuration_file, configuration.trim()).map_err(Error::WritingConfiguration)?;
        tracing::info!("Configuration file created successfully!");
    } else {
        tracing::warn!("Configuration not saved.");
    }

    Ok(())
}

/// Formats a vector of strings as quoted, comma-separated values for TOML.
///
/// # Arguments
///
/// * `items` - Vector of strings to format
///
/// # Returns
///
/// A string of quoted, comma-separated values
fn quote_format_strings(items: Vec<String>) -> String {
    items.iter().filter(|p| !p.is_empty()).map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", ")
}
