use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use colored::Colorize;

use mago_database::DatabaseReader;
use mago_linter::integration::IntegrationSet;
use mago_linter::registry::RuleRegistry;
use mago_linter::rule::AnyRule;
use mago_linter::settings::RulesSettings;
use mago_linter::settings::Settings;
use mago_reporting::Level;

use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::database;
use crate::error::Error;
use crate::pipeline::lint::LintContext;
use crate::pipeline::lint::LintMode;
use crate::pipeline::lint::run_lint_pipeline;

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
    /// Lint specific files or directories, overriding the source configuration.
    #[arg(help = "Lint specific files or directories, overriding the source configuration")]
    pub path: Vec<PathBuf>,

    /// Perform only parsing and semantic checks.
    #[arg(
        long,
        short = 's',
        help = "Perform only parsing and semantic checks",
        conflicts_with_all = ["list_rules", "explain", "only"],
    )]
    pub semantics: bool,

    #[arg(
        long,
        help = "Enable all linter rules for the most exhaustive analysis possible. This overrides your configuration, ignores PHP version constraints, and enables rules that are disabled by default. It is extremely noisy and not recommended for general use."
    )]
    pub pedantic: bool,

    #[arg(
        long,
        help = "Provide documentation for a specific linter rule, e.g. 'prefer-while-loop'",
        conflicts_with_all = ["list_rules", "sort", "fixable_only", "semantics", "reporting_target", "reporting_format"]
    )]
    pub explain: Option<String>,

    #[arg(
        long,
        help = "List all the enabled rules alongside their descriptions",
        conflicts_with_all = ["explain", "sort", "fixable_only", "semantics", "reporting_target", "reporting_format"]
    )]
    pub list_rules: bool,

    #[arg(
        long,
        help = "Output rule information in JSON format for documentation purposes, requires --list-rules",
        requires = "list_rules"
    )]
    pub json: bool,

    #[arg(short, long, help = "Specify rules to run, overriding the configuration file", conflicts_with = "semantics")]
    pub only: Vec<String>,

    #[clap(flatten)]
    pub reporting: ReportingArgs,
}

impl LintCommand {
    pub fn execute(self, mut configuration: Configuration, should_use_colors: bool) -> Result<ExitCode, Error> {
        configuration.source.excludes.extend(std::mem::take(&mut configuration.linter.excludes));

        let database = if !self.path.is_empty() {
            database::load_from_paths(&mut configuration.source, self.path, None)?
        } else {
            database::load_from_configuration(&mut configuration.source, false, None)?
        };

        let registry = if self.pedantic {
            RuleRegistry::build(
                Settings {
                    php_version: configuration.php_version,
                    integrations: IntegrationSet::all(),
                    rules: RulesSettings::default(),
                },
                None,
                true, // Include disabled rules.
            )
        } else {
            RuleRegistry::build(
                Settings {
                    php_version: configuration.php_version,
                    integrations: IntegrationSet::from_slice(&configuration.linter.integrations),
                    rules: configuration.linter.rules.clone(),
                },
                if self.only.is_empty() { None } else { Some(&self.only) },
                false,
            )
        };

        if let Some(explain_code) = self.explain {
            return explain_rule(&registry, &explain_code);
        }

        if self.list_rules {
            return list_rules(registry.rules(), self.json);
        }

        if database.is_empty() {
            tracing::info!("No files found to lint.");

            return Ok(ExitCode::SUCCESS);
        }

        let shared_context = LintContext {
            registry: Arc::new(registry),
            php_version: configuration.php_version,
            mode: if self.semantics { LintMode::SemanticsOnly } else { LintMode::Full },
        };

        let issues = run_lint_pipeline(database.read_only(), shared_context)?;

        self.reporting.process_issues(issues, configuration, should_use_colors, database)
    }
}

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
