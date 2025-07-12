use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use colored::Colorize;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_interner::ThreadedInterner;
use mago_linter::Linter;
use mago_linter::settings::RuleSettings;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_semantics::SemanticsChecker;
use mago_source::SourceCategory;
use mago_source::SourceIdentifier;
use mago_source::SourceManager;
use mago_syntax::parser::parse_source;

use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::config::linter::LinterLevel;
use crate::error::Error;
use crate::metadata::compile_codebase_for_sources;
use crate::source;
use crate::utils::indent_multiline;
use crate::utils::progress::ProgressBarTheme;
use crate::utils::progress::create_progress_bar;
use crate::utils::progress::remove_progress_bar;

#[derive(Parser, Debug)]
#[command(
    name = "lint",
    about = "Analyze and highlight issues in the project source code using configurable linting rules",
    long_about = r#"
The `lint` command is a powerful tool for analyzing your PHP codebase. By default, it performs
a full analysis, including parsing, semantic checks, and linting based on customizable rules.

This command is ideal for enforcing code quality standards, debugging issues, and maintaining
a consistent, clean codebase.

- Use no flags for a full lint check (parsing, semantic checks, reflection, compilation issues, and linting).
- Use `--semantics-only` or `-s` for a quick semantic check (parsing and semantic checks).
- Use `--compilation` or `-c` to include reflection and compilation issue checks in addition to semantic checks.
"#
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
        default_value_t = false,
        conflicts_with = "compilation"
    )]
    pub semantics_only: bool,

    /// Perform parsing, semantic checks, reflection and compilation issue checks.
    #[arg(
        long,
        short = 'c',
        help = "Perform parsing, semantic checks, reflection and compilation issue checks",
        default_value_t = false,
        conflicts_with = "semantics_only"
    )]
    pub compilation: bool,

    #[arg(
        long,
        help = "Provide documentation for a specific linter rule, e.g. 'consistency/lowercase-hint'",
        conflicts_with = "list_rules",
        conflicts_with = "sort",
        conflicts_with = "fixable_only",
        conflicts_with = "semantics_only",
        conflicts_with = "compilation",
        conflicts_with = "reporting_target",
        conflicts_with = "reporting_format"
    )]
    pub explain: Option<String>,

    #[arg(
        long,
        help = "List all the enabled rules alongside their descriptions",
        conflicts_with = "explain",
        conflicts_with = "sort",
        conflicts_with = "fixable_only",
        conflicts_with = "semantics_only",
        conflicts_with = "compilation",
        conflicts_with = "reporting_target",
        conflicts_with = "reporting_format"
    )]
    pub list_rules: bool,

    #[arg(
        short,
        long,
        help = "Do not load default plugins, only load the ones specified in the configuration.",
        conflicts_with = "compilation",
        conflicts_with = "semantics_only"
    )]
    pub no_default_plugins: bool,

    #[arg(
        short,
        long,
        help = "Specify plugins to load, overriding the configuration.",
        conflicts_with = "compilation",
        conflicts_with = "semantics_only"
    )]
    pub plugins: Vec<String>,

    #[clap(flatten)]
    pub reporting: ReportingArgs,
}

pub async fn execute(command: LintCommand, mut configuration: Configuration) -> Result<ExitCode, Error> {
    let interner = ThreadedInterner::new();

    if command.no_default_plugins {
        configuration.linter.default_plugins = Some(false);
    }

    if !command.plugins.is_empty() {
        configuration.linter.plugins = command.plugins;
    }

    if let Some(rule) = &command.explain {
        return explain_rule(&interner, rule, &configuration);
    }

    if command.list_rules {
        return list_rules(&interner, &configuration);
    }

    // Load sources
    let source_manager = if !command.path.is_empty() {
        source::from_paths(&interner, &configuration.source, command.path, !command.semantics_only).await?
    } else {
        source::load(&interner, &configuration.source, !command.semantics_only, !command.semantics_only).await?
    };

    let issues = if command.semantics_only {
        get_semantics_issues_only(&interner, &source_manager, configuration.php_version).await?
    } else if command.compilation {
        get_compilation_and_semantic_issues(&interner, &source_manager, configuration.php_version).await?
    } else {
        lint_codebase(&interner, &source_manager, &configuration).await?
    };

    command.reporting.process_issues(issues, configuration, interner, source_manager).await
}

#[inline]
pub(super) fn create_linter(
    interner: &ThreadedInterner,
    configuration: &Configuration,
    codebase: CodebaseMetadata,
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
#[inline]
pub(super) fn explain_rule(
    interner: &ThreadedInterner,
    rule: &str,
    configuration: &Configuration,
) -> Result<ExitCode, Error> {
    let linter = create_linter(interner, configuration, CodebaseMetadata::new());
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
                    println!("name = \"{rule}\"");
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
                    println!("name = \"{rule}\"");
                    print!("{}", toml::to_string(&usage.options).map_err(Error::from)?.yellow());
                    println!("{}", "```".dimmed());
                }
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

#[inline]
pub(super) fn list_rules(interner: &ThreadedInterner, configuration: &Configuration) -> Result<ExitCode, Error> {
    let linter = create_linter(interner, configuration, CodebaseMetadata::new());
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
        println!("    - {description}");
        println!();
        println!("    {}", footer.bright_black());
    }

    Ok(ExitCode::SUCCESS)
}

#[inline]
pub(super) async fn get_semantics_issues_only(
    interner: &ThreadedInterner,
    manager: &SourceManager,
    php_version: PHPVersion,
) -> Result<IssueCollection, Error> {
    // Collect all user-defined sources.
    let sources: Vec<_> = manager.source_ids_for_category(SourceCategory::UserDefined);
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
                let name_resolver = NameResolver::new(&interner);
                let semantics_checker = SemanticsChecker::new(&php_version, &interner);
                let (program, parse_error) = parse_source(&interner, &source);
                let resolved_names = name_resolver.resolve(&program);

                let mut semantic_issues = semantics_checker.check(&source, &program, &resolved_names);
                if let Some(error) = &parse_error {
                    semantic_issues.push(Into::<Issue>::into(error));
                }

                progress_bar.inc(1);

                Result::<_, Error>::Ok(semantic_issues)
            }
        }));
    }

    let mut results = Vec::with_capacity(length);

    for handle in handles {
        let issue_collection = handle.await??;

        results.extend(issue_collection);
    }

    remove_progress_bar(progress_bar);

    Ok(IssueCollection::from(results.into_iter()))
}

#[inline]
pub(super) async fn get_compilation_and_semantic_issues(
    interner: &ThreadedInterner,
    manager: &SourceManager,
    php_version: PHPVersion,
) -> Result<IssueCollection, Error> {
    // Compile the codebase.
    let mut codebase = compile_codebase_for_sources(manager, &mut SymbolReferences::new(), interner).await?;

    // Collect all semantics issues only.
    let mut issues = get_semantics_issues_only(interner, manager, php_version).await?;

    // Add the compilation issues.
    issues.extend(codebase.take_issues(true));

    Ok(issues)
}

#[inline]
pub(super) async fn lint_codebase(
    interner: &ThreadedInterner,
    manager: &SourceManager,
    configuration: &Configuration,
) -> Result<IssueCollection, Error> {
    let php_version = configuration.php_version;
    let sources: Vec<_> = manager.source_ids_for_category(SourceCategory::UserDefined);
    let length = sources.len();

    let mut codebase = compile_codebase_for_sources(manager, &mut SymbolReferences::new(), interner).await?;
    let mut result = codebase.take_issues(true);

    let linter = create_linter(interner, configuration, codebase);
    let lint_progress = create_progress_bar(length, "🧹  Linting", ProgressBarTheme::Red);
    let mut handles = Vec::with_capacity(length);
    for source_id in sources {
        handles.push(tokio::spawn({
            let interner = interner.clone();
            let manager = manager.clone();
            let linter = linter.clone();
            let lint_progress = lint_progress.clone();

            async move {
                let result = lint_single_source(&manager, &interner, &linter, &php_version, source_id);
                lint_progress.inc(1);

                result
            }
        }));
    }

    for handle in handles {
        result.extend(handle.await??);
    }

    remove_progress_bar(lint_progress);

    Ok(result)
}

fn lint_single_source(
    source_manager: &SourceManager,
    interner: &ThreadedInterner,
    linter: &Linter,
    php_version: &PHPVersion,
    source_id: SourceIdentifier,
) -> Result<Vec<Issue>, Error> {
    let source = source_manager.load(&source_id)?;

    let name_resolver = NameResolver::new(interner);
    let semantics_checker = SemanticsChecker::new(php_version, interner);

    let (program, parsing_error) = parse_source(interner, &source);
    let resolved_names = name_resolver.resolve(&program);

    let mut issues = Vec::new();
    if let Some(error) = &parsing_error {
        issues.push(Into::<Issue>::into(error));
    }

    issues.extend(semantics_checker.check(&source, &program, &resolved_names));
    issues.extend(linter.lint(&source, &program, &resolved_names));

    Ok(issues)
}
