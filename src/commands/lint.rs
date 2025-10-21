//! Linting command implementation.
//!
//! This module implements the `mago lint` command, which checks PHP code against a
//! configurable set of linting rules to identify style violations, code smells, and
//! potential quality issues.
//!
//! # Features
//!
//! The linter provides several modes of operation:
//!
//! - **Full Linting** (default): Run all enabled rules against the codebase
//! - **Semantics Only** (`--semantics`): Only validate syntax and basic semantic structure
//! - **Pedantic Mode** (`--pedantic`): Enable all available rules for maximum thoroughness
//! - **Targeted Linting** (`--only`): Run only specific rules by code
//!
//! # Rule Discovery
//!
//! The command supports introspection of available rules:
//!
//! - **List Rules** (`--list-rules`): Display all currently enabled rules
//! - **Explain Rule** (`--explain <CODE>`): Show detailed documentation for a specific rule
//!
//! # Baseline Support
//!
//! The linter integrates with baseline functionality to enable incremental adoption
//! by ignoring pre-existing issues while catching new ones. See [`BaselineReportingArgs`]
//! for baseline options.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;
use colored::Colorize;

use mago_database::DatabaseReader;
use mago_linter::registry::RuleRegistry;
use mago_linter::rule::AnyRule;
use mago_orchestrator::service::lint::LintMode;
use mago_reporting::Level;

use crate::commands::args::baseline_reporting::BaselineReportingArgs;
use crate::config::Configuration;
use crate::error::Error;
use crate::utils::create_orchestrator;

/// Command for linting PHP source code.
///
/// This command runs configurable linting rules to check code style, consistency,
/// and quality. It supports multiple modes including semantic-only validation,
/// pedantic checking, and targeted rule execution.
///
/// # Examples
///
/// Lint all configured source paths:
/// ```text
/// mago lint
/// ```
///
/// Lint specific files or directories:
/// ```text
/// mago lint src/ tests/
/// ```
///
/// Run only specific rules:
/// ```text
/// mago lint --only no-empty,constant-condition
/// ```
///
/// Show documentation for a rule:
/// ```text
/// mago lint --explain no-empty
/// ```
#[derive(Parser, Debug)]
#[command(
    name = "lint",
    about = "Lints PHP source code for style, consistency, and structural errors.",
    long_about = indoc::indoc! {"
        Analyzes PHP files to find and report stylistic issues, inconsistencies, and
        potential code quality improvements based on a configurable set of rules.

        This is the primary tool for ensuring your codebase adheres to established
        coding standards and best practices.

        USAGE:

            mago lint
            mago lint src/
            mago lint --list-rules
            mago lint --explain no-empty
            mago lint --only no-empty,constant-condition

        By default, it lints all source paths defined in your `mago.toml` file. You can
        also provide specific file or directory paths to lint a subset of your project.
    "}
)]
pub struct LintCommand {
    /// Specific files or directories to lint instead of using configuration.
    ///
    /// When provided, these paths override the source configuration in mago.toml.
    /// You can specify individual files or entire directories to lint.
    #[arg()]
    pub path: Vec<PathBuf>,

    /// Skip linter rules and only perform basic syntax and semantic validation.
    ///
    /// This mode only checks that your PHP code parses correctly and has valid
    /// semantic structure, without applying any style or quality rules.
    /// Useful for quick syntax validation.
    #[arg(long, short = 's', conflicts_with_all = ["list_rules", "explain", "only"])]
    pub semantics: bool,

    /// Enable every available linter rule for maximum thoroughness.
    ///
    /// This overrides your configuration and enables all rules, including those
    /// disabled by default. The output will be extremely verbose and is not
    /// recommended for regular use. Useful for comprehensive code audits.
    #[arg(long)]
    pub pedantic: bool,

    /// Show detailed documentation for a specific linter rule.
    ///
    /// Displays the rule's description, examples of good and bad code,
    /// and available configuration options. Use the rule's code name,
    /// such as 'no-empty' or 'prefer-while-loop'.
    #[arg(
        long,
        conflicts_with_all = ["list_rules", "sort", "fixable_only", "semantics", "reporting_target", "reporting_format"]
    )]
    pub explain: Option<String>,

    /// Show all currently enabled linter rules and their descriptions.
    ///
    /// This displays a table of all rules that are active for your current
    /// configuration, along with their severity levels and categories.
    /// Combine with --json for machine-readable output.
    #[arg(
        long,
        conflicts_with_all = ["explain", "sort", "fixable_only", "semantics", "reporting_target", "reporting_format"]
    )]
    pub list_rules: bool,

    /// Output rule information in JSON format.
    ///
    /// When combined with --list-rules, outputs rule information as JSON
    /// instead of a human-readable table. Useful for generating documentation
    /// or integrating with other tools.
    #[arg(long, requires = "list_rules")]
    pub json: bool,

    /// Run only specific rules, ignoring the configuration file.
    ///
    /// Provide a comma-separated list of rule codes to run only those rules.
    /// This overrides your mago.toml configuration and is useful for targeted
    /// analysis or testing specific rules.
    #[arg(short, long, conflicts_with = "semantics")]
    pub only: Vec<String>,

    #[clap(flatten)]
    pub baseline_reporting: BaselineReportingArgs,
}

impl LintCommand {
    /// Executes the lint command.
    ///
    /// This method orchestrates the linting process based on the command's configuration:
    ///
    /// 1. Creates an orchestrator with the configured settings
    /// 2. Applies path overrides if `path` was provided
    /// 3. Loads the database by scanning the file system
    /// 4. Creates the linting service with the database
    /// 5. Handles special modes (`--explain`, `--list-rules`)
    /// 6. Runs linting in the appropriate mode (full or semantics-only)
    /// 7. Processes and reports issues through the baseline processor
    ///
    /// # Arguments
    ///
    /// * `configuration` - The loaded configuration containing linter settings
    /// * `color_choice` - Whether to use colored output
    ///
    /// # Returns
    ///
    /// - `Ok(ExitCode::SUCCESS)` if linting completed successfully (even with issues found)
    /// - `Ok(ExitCode::FAILURE)` if a rule explanation was requested but the rule wasn't found
    /// - `Err(Error)` if database loading, linting, or reporting failed
    ///
    /// # Special Modes
    ///
    /// - **Explain Mode** (`--explain`): Displays detailed rule documentation and exits
    /// - **List Mode** (`--list-rules`): Shows all enabled rules and exits
    /// - **Empty Database**: Logs a message and exits successfully if no files found
    pub fn execute(self, configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        let mut orchestrator = create_orchestrator(&configuration, color_choice, self.pedantic);
        orchestrator.add_exclude_patterns(configuration.linter.excludes.iter());
        if !self.path.is_empty() {
            orchestrator.set_source_paths(self.path.iter());
        }

        let database = orchestrator.load_database(&configuration.source.workspace, false, None)?;
        let service = orchestrator.get_lint_service(database.read_only());

        if let Some(explain_code) = self.explain {
            let registry = service.create_registry(
                if self.only.is_empty() { None } else { Some(&self.only) },
                self.pedantic, // Enable all rules if pedantic is set
            );

            return explain_rule(&registry, &explain_code);
        }

        if self.list_rules {
            let registry = service.create_registry(
                if self.only.is_empty() { None } else { Some(&self.only) },
                self.pedantic, // Enable all rules if pedantic is set
            );

            return list_rules(registry.rules(), self.json);
        }

        if database.is_empty() {
            tracing::info!("No files found to lint.");

            return Ok(ExitCode::SUCCESS);
        }

        let issues = service.lint(if self.semantics { LintMode::SemanticsOnly } else { LintMode::Full })?;

        let baseline = configuration.linter.baseline.as_deref();
        let processor = self.baseline_reporting.get_processor(orchestrator, database, color_choice, baseline);

        processor.process_issues(issues)
    }
}

/// Displays detailed documentation for a specific linting rule.
///
/// This function shows comprehensive information about a rule including its
/// description, code, category, and good/bad examples. The output is formatted
/// for terminal display with colors and proper wrapping.
///
/// # Arguments
///
/// * `registry` - The rule registry containing all available rules
/// * `code` - The rule code to explain (e.g., "no-empty", "prefer-while-loop")
///
/// # Returns
///
/// - `Ok(ExitCode::SUCCESS)` if the rule was found and explained
/// - `Ok(ExitCode::FAILURE)` if the rule code doesn't exist in the registry
///
/// # Output Format
///
/// The explanation includes:
/// - Rule name and description (wrapped to 80 characters)
/// - Rule code and category
/// - Good example (if available)
/// - Bad example (if available)
/// - Suggested command to try the rule
pub fn explain_rule(registry: &RuleRegistry, code: &str) -> Result<ExitCode, Error> {
    let Some(rule) = registry.rules().iter().find(|r| r.meta().code == code) else {
        println!();
        println!("  {}", "Error: Rule not found".red().bold());
        println!("  {}", format!("Could not find a rule with the code '{}'.", code).bright_black());
        println!("  {}", "Please check the spelling and try again.".bright_black());
        println!();

        return Ok(ExitCode::FAILURE);
    };

    let meta = rule.meta();

    println!();
    println!("  ╭─ {} {}", "Rule".bold(), meta.name.cyan().bold());
    println!("  │");

    println!("{}", wrap_and_prefix(meta.description, "  │  ", 80));

    println!("  │");
    println!("  │  {}: {}", "Code".bold(), meta.code.yellow());
    println!("  │  {}: {}", "Category".bold(), meta.category.as_str().magenta());

    if !meta.good_example.trim().is_empty() {
        println!("  │");
        println!("  │  {}", "✅ Good Example".green().bold());
        println!("  │");
        println!("{}", colorize_code_block(meta.good_example));
    }

    if !meta.bad_example.trim().is_empty() {
        println!("  │");
        println!("  │  {}", "🚫 Bad Example".red().bold());
        println!("  │");
        println!("{}", colorize_code_block(meta.bad_example));
    }

    println!("  │");
    println!("  │  {}", "Try it out!".bold());
    println!("  │    {}", format!("mago lint --only {}", meta.code).bright_black());
    println!("  ╰─");
    println!();

    Ok(ExitCode::SUCCESS)
}

/// Lists all enabled linting rules.
///
/// This function displays all currently active rules in either human-readable
/// table format or JSON format. The table is formatted with proper column
/// alignment showing rule name, code, severity level, and category.
///
/// # Arguments
///
/// * `rules` - The list of enabled rules to display
/// * `json` - Whether to output in JSON format instead of a table
///
/// # Returns
///
/// Always returns `Ok(ExitCode::SUCCESS)` since listing rules cannot fail.
///
/// # Output Formats
///
/// **Table Format** (default):
/// - Aligned columns for Name, Code, Level, and Category
/// - Color-coded severity levels (Error=red, Warning=yellow, etc.)
/// - Helpful footer with `--explain` command suggestion
///
/// **JSON Format** (`--json`):
/// - Array of rule metadata objects
/// - Pretty-printed for readability
/// - Machine-parseable for tooling integration
pub fn list_rules(rules: &[AnyRule], json: bool) -> Result<ExitCode, Error> {
    if rules.is_empty() && !json {
        println!("{}", "No rules are currently enabled.".yellow());

        return Ok(ExitCode::SUCCESS);
    }

    if json {
        let metas: Vec<_> = rules.iter().map(|r| r.meta()).collect();
        println!("{}", serde_json::to_string_pretty(&metas)?);

        return Ok(ExitCode::SUCCESS);
    }

    let max_name = rules.iter().map(|r| r.meta().name.len()).max().unwrap_or(0);
    let max_code = rules.iter().map(|r| r.meta().code.len()).max().unwrap_or(0);

    println!();
    println!(
        "  {: <width_name$}   {: <width_code$}   {: <8}   {}",
        "Name".bold().underline(),
        "Code".bold().underline(),
        "Level".bold().underline(),
        "Category".bold().underline(),
        width_name = max_name,
        width_code = max_code,
    );
    println!();

    for rule in rules {
        let meta = rule.meta();
        let level_str = match rule.default_level() {
            Level::Error => "Error".red(),
            Level::Warning => "Warning".yellow(),
            Level::Help => "Help".green(),
            Level::Note => "Note".blue(),
        };

        println!(
            "  {: <width_name$}   {: <width_code$}   {: <8}   {}",
            meta.name.cyan(),
            meta.code.bright_black(),
            level_str.bold(),
            meta.category.as_str().magenta(),
            width_name = max_name,
            width_code = max_code,
        );
    }

    println!();
    println!("  Run {} to see more information about a specific rule.", "mago lint --explain <CODE>".bold());
    println!();

    Ok(ExitCode::SUCCESS)
}

fn colorize_code_block(code: &str) -> String {
    let mut colored_code = String::new();
    for line in code.trim().lines() {
        let trimmed_line = line.trim_start();
        let indentation = &line[..line.len() - trimmed_line.len()];

        let colored_line =
            if trimmed_line.starts_with("<?php") || trimmed_line.starts_with("<?") || trimmed_line.starts_with("?>") {
                trimmed_line.yellow().bold().to_string()
            } else {
                trimmed_line.to_string()
            };

        colored_code.push_str(&format!("  │    {}{}\n", indentation.bright_black(), colored_line));
    }

    colored_code.trim_end().to_string()
}

fn wrap_and_prefix(text: &str, prefix: &str, width: usize) -> String {
    let mut result = String::new();
    let wrap_width = width.saturating_sub(prefix.len());

    for (i, paragraph) in text.trim().split("\n\n").enumerate() {
        if i > 0 {
            result.push_str(prefix);
            result.push('\n');
        }

        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if !current_line.is_empty() && current_line.len() + word.len() + 1 > wrap_width {
                result.push_str(prefix);
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
            }

            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }

        if !current_line.is_empty() {
            result.push_str(prefix);
            result.push_str(&current_line);
            result.push('\n');
        }
    }

    result.trim_end().to_string()
}
