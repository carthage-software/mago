use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use clap::Parser;
use colored::Colorize;
use dialoguer::Confirm;
use dialoguer::Input;
use dialoguer::MultiSelect;
use dialoguer::console::style;
use dialoguer::theme::ColorfulTheme;

use mago_composer::AutoloadPsr4value;
use mago_composer::ComposerPackage;
use mago_composer::ComposerPackageAutoloadDevPsr4value;
use mago_linter::integration::Integration;
use mago_php_version::PHPVersion;

use crate::config::Configuration;
use crate::consts::COMPOSER_JSON_FILE;
use crate::consts::CONFIGURATION_FILE_NAME;
use crate::consts::DEFAULT_PHP_VERSION;
use crate::error::Error;
use crate::utils::version::extract_minimum_php_version;

const CONFIGURATION_TEMPLATE: &str = r#"# Welcome to Mago!
# For full documentation, see https://mago.carthage.software/tools/overview
php-version = "{php_version}"

[source]
paths = [{paths}]
includes = [{includes}]
excludes = [{excludes}]

[formatter]
print-width = {print_width}
tab-width = {tab_width}
use-tabs = {use_tabs}

[linter]
integrations = [{integrations}]

[linter.rules]
ambiguous-function-call = { enabled = false }
literal-named-argument = { enabled = false }
halstead = { effort-threshold = 7000 }

[analyzer]
{analyzer_settings}
"#;

#[derive(Parser, Debug)]
#[command(
    name = "init",
    about = "Initialize Mago for your project with a guided setup.",
    long_about = "Creates a new mago.toml configuration file by walking you through a setup process."
)]
pub struct InitCommand;

pub fn execute(
    _: InitCommand,
    configuration: Configuration,
    configuration_file: Option<PathBuf>,
) -> Result<ExitCode, Error> {
    let theme = ColorfulTheme {
        prompt_prefix: style("".to_string()),
        success_prefix: style("".to_string()),
        error_prefix: style("".to_string()),
        ..Default::default()
    };

    let configuration_file =
        configuration_file.unwrap_or_else(|| configuration.source.workspace.join(CONFIGURATION_FILE_NAME));

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
    let (print_width, tab_width, use_tabs) = setup_formatter(&theme)?;
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
        .replace("{print_width}", &print_width.to_string())
        .replace("{tab_width}", &tab_width.to_string())
        .replace("{use_tabs}", &use_tabs.to_string())
        .replace("{analyzer_settings}", &build_analyzer_settings_string(&analyzer_settings));

    if write_configuration_if_confirmed(&theme, &configuration_file, &config_content)? {
        print_final_summary();
    } else {
        println!("  ❌ {}", "Initialization cancelled. No file was written.".yellow());
        println!();
    }

    Ok(ExitCode::SUCCESS)
}

#[derive(Debug)]
struct InitializationProjectSettings {
    php_version: String,
    paths: Vec<String>,
    includes: Vec<String>,
    excludes: Vec<String>,
}

#[derive(Debug)]
struct InitializationAnalyzerSettings {
    ignore: Vec<String>,
    disabled_categories: Vec<String>,
    find_unused_definitions: bool,
    find_unused_expressions: bool,
    analyze_dead_code: bool,
    check_throws: bool,
    allow_possibly_undefined_array_keys: bool,
    perform_heuristic_checks: bool,
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

fn setup_formatter(theme: &ColorfulTheme) -> Result<(u16, u8, bool), Error> {
    print_step_header(3, "Formatter Configuration");
    println!("  │  {}", "The Formatter automatically rewrites your files to a consistent style.".bright_black());
    println!("  │  {}", "This ends debates about spacing and helps you focus on the code.".bright_black());
    println!("  │");

    let defaults = (120, 4, false);

    if Confirm::with_theme(theme)
        .with_prompt(" │  The default settings are PSR-12 compatible. Do you want to customize them?")
        .default(false)
        .interact()?
    {
        println!("  │");
        let print_width = prompt_for_u16(theme, " │  Print width (line length)", defaults.0)?;
        let tab_width = prompt_for_u8(theme, " │  Tab width (spaces)", defaults.1)?;
        let use_tabs =
            Confirm::with_theme(theme).with_prompt(" │  Use tabs instead of spaces?").default(defaults.2).interact()?;
        println!("  │");
        println!("  │  {}", "ℹ️  The formatter has many more options. Check the docs to customize it further.".blue());
        println!("  ╰─");
        Ok((print_width, tab_width, use_tabs))
    } else {
        println!("  │");
        println!("  │  {}", "Great choice! Sticking to the defaults is highly recommended.".green());
        println!("  ╰─");
        Ok(defaults)
    }
}

fn setup_analyzer(theme: &ColorfulTheme) -> Result<InitializationAnalyzerSettings, Error> {
    print_step_header(4, "Analyzer Configuration");
    println!("  │  {}", "The Analyzer finds logical bugs and type errors before you run your code.".bright_black());
    println!("  │  {}", "This is the most powerful part of Mago.".bright_black());
    println!("  │");

    if Confirm::with_theme(theme)
        .with_prompt(" │  Would you like to go through the advanced setup for the analyzer?")
        .default(false)
        .interact()?
    {
        println!("  │");
        let find_unused_definitions = Confirm::with_theme(theme)
            .with_prompt(" │  Find unused definitions (e.g., private methods)?")
            .default(true)
            .interact()?;
        let find_unused_expressions = Confirm::with_theme(theme)
            .with_prompt(" │  Find unused expressions (e.g., `$a + $b;`)?")
            .default(false)
            .interact()?;
        let analyze_dead_code = Confirm::with_theme(theme)
            .with_prompt(" │ Analyze code that appears to be unreachable?")
            .default(false)
            .interact()?;
        let check_throws = Confirm::with_theme(theme)
            .with_prompt(" │  Check for unhandled thrown exceptions?")
            .default(true)
            .interact()?;
        let allow_possibly_undefined_array_keys = Confirm::with_theme(theme)
            .with_prompt(" │  Allow accessing possibly undefined array keys?")
            .default(true)
            .interact()?;
        let perform_heuristic_checks = Confirm::with_theme(theme)
            .with_prompt(" │  Enable extra heuristic checks for code quality?")
            .default(true)
            .interact()?;

        println!("  │");
        let ignore = prompt_for_paths(theme, "Issue codes to ignore globally (comma-separated)", None)?;
        let disabled_categories = prompt_for_disabled_categories(theme)?;

        println!("  ╰─");
        Ok(InitializationAnalyzerSettings {
            ignore,
            disabled_categories,
            find_unused_definitions,
            find_unused_expressions,
            analyze_dead_code,
            check_throws,
            allow_possibly_undefined_array_keys,
            perform_heuristic_checks,
        })
    } else {
        println!("  │");
        println!("  │  {}", "Let's enable the most common features:".bright_black());
        println!("  │");
        let find_unused_definitions =
            Confirm::with_theme(theme).with_prompt(" │  Find unused definitions?").default(true).interact()?;
        let check_throws = Confirm::with_theme(theme)
            .with_prompt(" │  Check for unhandled thrown exceptions?")
            .default(true)
            .interact()?;
        let perform_heuristic_checks = Confirm::with_theme(theme)
            .with_prompt(" │  Enable extra heuristic checks for code quality?")
            .default(true)
            .interact()?;
        println!("  ╰─");
        Ok(InitializationAnalyzerSettings {
            ignore: Vec::new(),
            disabled_categories: Vec::new(),
            find_unused_definitions,
            find_unused_expressions: false,
            analyze_dead_code: false,
            check_throws,
            allow_possibly_undefined_array_keys: true,
            perform_heuristic_checks,
        })
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
    integrations
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
    let items = &[Integration::Psl, Integration::Laravel, Integration::PHPUnit, Integration::Symfony];
    let selections = MultiSelect::with_theme(theme)
        .with_prompt(" │  Select integrations to enable (space to select, enter to confirm)")
        .items(items)
        .interact()?;
    Ok(selections.into_iter().map(|i| items[i]).collect())
}

fn prompt_for_disabled_categories(theme: &ColorfulTheme) -> Result<Vec<String>, Error> {
    const CATEGORIES: [&str; 19] = [
        "falsable",
        "nullable",
        "mixed",
        "redundancy",
        "reference",
        "unreachable",
        "deprecation",
        "impossibility",
        "ambiguity",
        "existence",
        "template",
        "argument",
        "operand",
        "property",
        "generator",
        "array",
        "return",
        "method",
        "iterator",
    ];

    let selections = MultiSelect::with_theme(theme)
        .with_prompt(" │  Select issue categories to disable (all are enabled by default)")
        .items(CATEGORIES)
        .interact()?;

    Ok(selections.into_iter().map(|i| CATEGORIES[i].to_string()).collect())
}

fn quote_format_strings(items: &[String]) -> String {
    items.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", ")
}

fn build_analyzer_settings_string(settings: &InitializationAnalyzerSettings) -> String {
    let mut lines = Vec::new();

    if !settings.ignore.is_empty() {
        lines.push("# Ignored Issues".to_string());
        lines.push(format!("ignore = [{}]", quote_format_strings(&settings.ignore)));
        lines.push("".to_string());
    }

    if !settings.disabled_categories.is_empty() {
        lines.push("# Disabled Issue Categories".to_string());
        for category in &settings.disabled_categories {
            lines.push(format!("{}-issues = false", category));
        }
        lines.push("".to_string());
    }

    if !lines.is_empty() {
        lines.push("# Analyzer Settings".to_string());
    }
    lines.push(format!("find-unused-definitions = {}", settings.find_unused_definitions));
    lines.push(format!("find-unused-expressions = {}", settings.find_unused_expressions));
    lines.push(format!("analyze-dead-code = {}", settings.analyze_dead_code));
    lines.push(format!("check-throws = {}", settings.check_throws));
    lines.push(format!("allow-possibly-undefined-array-keys = {}", settings.allow_possibly_undefined_array_keys));
    lines.push(format!("perform-heuristic-checks = {}", settings.perform_heuristic_checks));

    lines.join("\n")
}
