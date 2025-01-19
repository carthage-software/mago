use std::process::ExitCode;

use clap::Parser;
use colored::Colorize;

use mago_interner::ThreadedInterner;
use mago_linter::settings::RuleSettings;
use mago_linter::settings::Settings;
use mago_linter::Linter;
use mago_reflection::CodebaseReflection;
use mago_reflector::reflect;
use mago_reporting::reporter::Reporter;
use mago_reporting::reporter::ReportingFormat;
use mago_reporting::reporter::ReportingTarget;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_semantics::Semantics;
use mago_source::error::SourceError;
use mago_source::SourceCategory;
use mago_source::SourceManager;

use crate::config::linter::LinterLevel;
use crate::config::Configuration;
use crate::enum_variants;
use crate::error::Error;
use crate::reflection::reflect_all_non_user_defined_sources;
use crate::source;
use crate::utils::indent_multiline;
use crate::utils::progress::create_progress_bar;
use crate::utils::progress::remove_progress_bar;
use crate::utils::progress::ProgressBarTheme;

#[derive(Parser, Debug)]
#[command(
    name = "lint",
    about = "Analyze and highlight issues in the project source code using configurable linting rules",
    long_about = r#"
The `lint` command is a powerful tool for analyzing your PHP codebase. By default, it performs
a full analysis, including parsing, semantic checks, and linting based on customizable rules.

This command is ideal for enforcing code quality standards, debugging issues, and maintaining
a consistent, clean codebase. Use `--semantics-only` for a quick validation of code correctness
or the default mode for a comprehensive analysis.
"#
)]
pub struct LintCommand {
    /// Filter the output to only show issues that can be automatically fixed with `mago fix`.
    #[arg(long, short = 'f', help = "Filter the output to only show fixable issues", default_value_t = false)]
    pub fixable_only: bool,

    /// Perform only semantic analysis (parsing and semantic checks).
    #[arg(long, short = 's', help = "Only perform parsing and semantic checks", default_value_t = false)]
    pub semantics_only: bool,

    #[arg(long, help = "Provide documentation for a specific linter rule, e.g. 'consistency/lowercase-hint'")]
    pub explain: Option<String>,

    #[arg(long, help = "List all the enabled rules alongside their descriptions")]
    pub list_rules: bool,

    #[arg(long, help = "Sort the reported issues by level, code, and location")]
    pub sort: bool,

    /// Specify where the results should be reported.
    #[arg(
        long,
        default_value_t,
        help = "Specify where the results should be reported",
        ignore_case = true,
        value_parser = enum_variants!(ReportingTarget)
    )]
    pub reporting_target: ReportingTarget,

    /// Choose the format for reporting issues.
    #[arg(
        long,
        default_value_t,
        help = "Choose the format for reporting issues",
        ignore_case = true,
        value_parser = enum_variants!(ReportingFormat)
    )]
    pub reporting_format: ReportingFormat,
}

pub async fn execute(command: LintCommand, configuration: Configuration) -> Result<ExitCode, Error> {
    let interner = ThreadedInterner::new();
    let source_manager =
        source::load(&interner, &configuration.source, !command.semantics_only, !command.semantics_only).await?;

    if let Some(rule) = &command.explain {
        return explain_rule(&interner, rule, &configuration);
    }

    if command.list_rules {
        return list_rules(&interner, &configuration);
    }

    let mut issues = if command.semantics_only {
        check_sources(&interner, &source_manager).await?
    } else {
        lint_sources(&interner, &source_manager, &configuration).await?
    };

    let issues_contain_errors = issues.get_highest_level().is_some_and(|level| level <= Level::Error);

    let reporter = Reporter::new(interner, source_manager, command.reporting_target);

    if command.sort {
        issues = issues.sorted();
    }

    if command.fixable_only {
        reporter.report(issues.only_fixable(), command.reporting_format)?;
    } else {
        reporter.report(issues, command.reporting_format)?;
    }

    Ok(if issues_contain_errors { ExitCode::FAILURE } else { ExitCode::SUCCESS })
}

pub(super) fn create_linter(
    interner: &ThreadedInterner,
    configuration: &Configuration,
    codebase: CodebaseReflection,
) -> Linter {
    let mut settings = Settings::new(configuration.php_version);

    if let Some(default_plugins) = configuration.linter.default_plugins {
        settings = settings.with_default_plugins(default_plugins);
    }

    settings = settings.with_plugins(configuration.linter.plugins.clone());

    for rule in &configuration.linter.rules {
        let rule_settings = match rule.level {
            Some(linter_level) => match linter_level {
                LinterLevel::Off => RuleSettings::disabled(),
                LinterLevel::Help => RuleSettings::from_level(Some(Level::Help)),
                LinterLevel::Note => RuleSettings::from_level(Some(Level::Note)),
                LinterLevel::Warning => RuleSettings::from_level(Some(Level::Warning)),
                LinterLevel::Error => RuleSettings::from_level(Some(Level::Error)),
            },
            None => RuleSettings::enabled(),
        };

        settings = settings.with_rule(rule.name.clone(), rule_settings.with_options(rule.options.clone()));
    }

    let mut linter = Linter::new(settings, interner.clone(), codebase);

    mago_linter::foreach_plugin!(|plugin| {
        linter.add_plugin(plugin);
    });

    linter
}

/// Displays detailed information about a single lint rule, including its name,
/// description, recognized options, and valid/invalid usage examples.
///
/// The overall structure and text remain the same as before; we only add
/// color styling to improve readability.
pub(super) fn explain_rule(
    interner: &ThreadedInterner,
    rule: &str,
    configuration: &Configuration,
) -> Result<ExitCode, Error> {
    let linter = create_linter(interner, configuration, CodebaseReflection::new());
    let configured_rules = linter.get_configured_rules();

    // Attempt to locate the rule
    let Some(configured_rule) = configured_rules.iter().find(|configured_rule| configured_rule.slug == rule) else {
        tracing::error!("Cannot find rule definition for '{}' in the linter configuration.", rule.bold().bright_red());
        tracing::error!("Please check the spelling and ensure the rule is enabled in the configuration.");

        return Ok(ExitCode::FAILURE);
    };

    let rule_definition = configured_rule.rule.get_definition();

    let title = format!("# {}", rule_definition.name);

    println!("{}", title.bold().underline());
    println!();
    println!("{}", rule_definition.description);

    println!("{}:", "## PHP Version".bold().underline());
    println!();
    if let Some(minimum_supported_php_version) = rule_definition.minimum_supported_php_version {
        println!(
            "{} {} {}",
            "- This rule requires PHP version".dimmed(),
            minimum_supported_php_version.to_string().bold().green(),
            "or higher.".dimmed()
        );
    } else {
        println!("{}", "- This rule does not have any minimum PHP version requirements.".dimmed());
    }

    if let Some(maximum_supported_php_version) = rule_definition.maximum_supported_php_version {
        println!(
            "{} {} {}",
            "- This rule supports PHP versions up to".dimmed(),
            maximum_supported_php_version.to_string().bold().green(),
            "exclusive.".dimmed()
        );
    } else {
        println!("{}", "- This rule does not have any maximum PHP version requirements.".dimmed());
    }

    if !rule_definition.options.is_empty() {
        println!();
        println!("{}:", "## Configuration Options".bold().underline());

        for option in &rule_definition.options {
            let title = format!("{} `{}`", "###".bold(), option.name);

            println!();
            println!("{}", title.yellow());
            println!();
            println!("{}", option.description);
            println!();
            println!("{}: {}", "Type".bold(), option.r#type.bright_magenta());
            println!("{}: {}", "Default".bold(), option.default.to_string().bright_magenta());
        }
    }

    if !rule_definition.examples.is_empty() {
        println!();

        let correct_usages = rule_definition.examples.iter().filter(|ex| ex.valid).collect::<Vec<_>>();
        let incorrect_usages = rule_definition.examples.iter().filter(|ex| !ex.valid).collect::<Vec<_>>();

        if !correct_usages.is_empty() {
            println!("{}:", "## Correct Usage Examples".bold().underline());

            for usage in correct_usages {
                println!();
                println!("{}: {} ✅", "### Example".bold().underline(), usage.description.bright_white());

                println!();
                println!("{}", "```php".dimmed());
                print!("{}", usage.snippet);
                println!("{}", "```".dimmed());

                if !usage.options.is_empty() {
                    println!();
                    println!("{}{}", "Configuration".blue().underline(), ":".blue());
                    println!();
                    println!("{}", "```toml".dimmed());
                    println!("[[linter.rules]]");
                    println!("name = \"{}\"", rule);
                    print!("{}", toml::to_string(&usage.options).map_err(Error::from)?.yellow());
                    println!("{}", "```".dimmed());
                }
            }

            if !incorrect_usages.is_empty() {
                println!();
            }
        }

        if !incorrect_usages.is_empty() {
            println!("{}:", "## Incorrect Usage Examples".bold().underline());

            for usage in incorrect_usages {
                println!();
                println!("{}: {} 🚫", "### Example".bold().underline(), usage.description.bright_white());

                println!();
                println!("{}", "```php".dimmed());
                print!("{}", usage.snippet);
                println!("{}", "```".dimmed());

                if !usage.options.is_empty() {
                    println!();
                    println!("{}{}", "Configuration".blue().underline(), ":".blue());
                    println!();
                    println!("{}", "```toml".dimmed());
                    println!("[[linter.rules]]");
                    println!("name = \"{}\"", rule);
                    print!("{}", toml::to_string(&usage.options).map_err(Error::from)?.yellow());
                    println!("{}", "```".dimmed());
                }
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

pub(super) fn list_rules(interner: &ThreadedInterner, configuration: &Configuration) -> Result<ExitCode, Error> {
    let linter = create_linter(interner, configuration, CodebaseReflection::new());
    let configured_rules = linter.get_configured_rules();
    if configured_rules.is_empty() {
        println!("{}", "No rules are currently configured or enabled.".bright_red());

        return Ok(ExitCode::SUCCESS);
    }

    println!("Listing {} rule(s):", configured_rules.len());
    for (i, configured_rule) in configured_rules.iter().enumerate() {
        let current_level = match linter.get_rule_level(&configured_rule.slug) {
            Some(level) => match level {
                Level::Error => "Error".red(),
                Level::Warning => "Warning".yellow(),
                Level::Help => "Help".green(),
                Level::Note => "Note".blue(),
            },
            None => {
                continue;
            }
        };

        let title = format!("{:2}. {}", i + 1, configured_rule.slug.underline());
        let description = indent_multiline(configured_rule.rule.get_definition().description, "      ", false);
        let explanation = format!("mago lint --explain {}", configured_rule.slug);
        let footer = format!("for more information, run {}", explanation.underline());

        println!();
        println!("{}", title.bold().bright_purple());
        println!();
        println!("    - Level: {}", current_level.bold());
        println!("    - {}", description);
        println!();
        println!("    {}", footer.bright_black());
    }

    Ok(ExitCode::SUCCESS)
}

#[inline]
pub(super) async fn lint_sources(
    interner: &ThreadedInterner,
    manager: &SourceManager,
    configuration: &Configuration,
) -> Result<IssueCollection, Error> {
    // Collect all user-defined sources.
    let sources: Vec<_> = manager.source_ids_for_category(SourceCategory::UserDefined).collect();
    let length = sources.len();

    let progress_bar = create_progress_bar(length, "🔎  Scanning", ProgressBarTheme::Yellow);
    let mut codebase = reflect_all_non_user_defined_sources(interner, manager).await?;
    let mut handles = Vec::with_capacity(length);
    for source_id in sources {
        handles.push(tokio::spawn({
            let interner = interner.clone();
            let manager = manager.clone();
            let progress_bar = progress_bar.clone();

            async move {
                // Step 1: load the source
                let source = manager.load(&source_id)?;
                // Step 2: build semantics
                let semantics = Semantics::build(&interner, source);
                let reflections = reflect(&interner, &semantics.source, &semantics.program, &semantics.names);
                progress_bar.inc(1);

                Result::<_, Error>::Ok((semantics, reflections))
            }
        }));
    }

    let mut semantics = Vec::with_capacity(length);
    for handle in handles {
        let (semantic, reflections) = handle.await??;

        codebase = mago_reflector::merge(interner, codebase, reflections);
        semantics.push(semantic);
    }

    mago_reflector::populate(interner, &mut codebase);

    remove_progress_bar(progress_bar);

    let linter = create_linter(interner, configuration, codebase);
    let progress_bar = create_progress_bar(length, "🧹  Linting", ProgressBarTheme::Red);
    let mut handles = Vec::with_capacity(length);
    for semantic in semantics {
        handles.push(tokio::spawn({
            let linter = linter.clone();
            let progress_bar = progress_bar.clone();

            async move {
                let mut issues = linter.lint(&semantic);
                issues.extend(semantic.issues);
                if let Some(error) = &semantic.parse_error {
                    issues.push(Into::<Issue>::into(error));
                }

                progress_bar.inc(1);

                Result::<_, SourceError>::Ok(issues)
            }
        }));
    }

    let mut results = Vec::with_capacity(length);
    for handle in handles {
        results.push(handle.await??);
    }

    remove_progress_bar(progress_bar);

    Ok(IssueCollection::from(results.into_iter().flatten()))
}

#[inline]
pub(super) async fn check_sources(
    interner: &ThreadedInterner,
    manager: &SourceManager,
) -> Result<IssueCollection, Error> {
    // Collect all user-defined sources.
    let sources: Vec<_> = manager.source_ids_for_category(SourceCategory::UserDefined).collect();
    let length = sources.len();

    let progress_bar = create_progress_bar(length, "🔎  Scanning", ProgressBarTheme::Magenta);
    let mut handles = Vec::with_capacity(length);
    for source_id in sources {
        handles.push(tokio::spawn({
            let interner = interner.clone();
            let manager = manager.clone();
            let progress_bar = progress_bar.clone();

            async move {
                let source = manager.load(&source_id)?;
                let semantics = Semantics::build(&interner, source);
                progress_bar.inc(1);

                Result::<_, Error>::Ok(semantics)
            }
        }));
    }

    let mut results = Vec::with_capacity(length);
    for handle in handles {
        let semantic = handle.await??;

        if let Some(error) = &semantic.parse_error {
            results.push(Into::<Issue>::into(error));
        }

        results.extend(semantic.issues);
    }

    remove_progress_bar(progress_bar);

    Ok(IssueCollection::from(results.into_iter()))
}
