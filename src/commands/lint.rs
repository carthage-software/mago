use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use colored::Colorize;

use mago_fixer::FixPlan;
use mago_fixer::SafetyClassification;
use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_interner::ThreadedInterner;
use mago_linter::Linter;
use mago_linter::settings::RuleSettings;
use mago_linter::settings::Settings;
use mago_php_version::PHPVersion;
use mago_project::Project;
use mago_project::ProjectBuilder;
use mago_project::module::Module;
use mago_project::module::ModuleBuildOptions;
use mago_reflection::CodebaseReflection;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_reporting::reporter::Reporter;
use mago_reporting::reporter::ReportingFormat;
use mago_reporting::reporter::ReportingTarget;
use mago_source::SourceCategory;
use mago_source::SourceIdentifier;
use mago_source::SourceManager;
use mago_source::error::SourceError;

use crate::config::Configuration;
use crate::config::linter::LinterLevel;
use crate::enum_variants;
use crate::error::Error;
use crate::reflection::reflect_non_user_sources;
use crate::source;
use crate::utils;
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

    /// Filter the output to only show issues that can be automatically fixed with `mago fix`.
    #[arg(
        long,
        short = 'f',
        help = "Filter the output to only show fixable issues",
        default_value_t = false,
        conflicts_with = "semantics_only",
        conflicts_with = "compilation"
    )]
    pub fixable_only: bool,

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
        long,
        help = "Sort the reported issues by level, code, and location",
        conflicts_with = "explain",
        conflicts_with = "list_rules"
    )]
    pub sort: bool,

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

    #[arg(
        long,
        help = "Apply fixes to the source code where possible.",
        conflicts_with = "semantics_only",
        conflicts_with = "compilation",
        conflicts_with = "fixable_only"
    )]
    pub fix: bool,

    /// Apply fixes that are marked as unsafe, including potentially unsafe fixes.
    #[arg(
        long,
        help = "Apply fixes marked as unsafe, including those with potentially destructive changes",
        requires = "fix"
    )]
    pub r#unsafe: bool,

    /// Apply fixes that are marked as potentially unsafe.
    #[arg(long, help = "Apply fixes marked as potentially unsafe, which may require manual review", requires = "fix")]
    pub potentially_unsafe: bool,

    #[arg(long, help = "Format the fixed files after applying the changes.", requires = "fix", alias = "fmt")]
    pub format: bool,

    /// Run the command without writing any changes to disk.
    #[arg(
        long,
        short = 'd',
        help = "Preview the fixes without applying them, showing what changes would be made",
        requires = "fix"
    )]
    pub dry_run: bool,

    /// Specify where the results should be reported.
    #[arg(
        long,
        default_value_t,
        help = "Specify where the results should be reported",
        ignore_case = true,
        value_parser = enum_variants!(ReportingTarget),
        conflicts_with = "explain",
        conflicts_with = "list_rules",
        conflicts_with = "fix",
    )]
    pub reporting_target: ReportingTarget,

    /// Choose the format for reporting issues.
    #[arg(
        long,
        default_value_t,
        help = "Choose the format for reporting issues",
        ignore_case = true,
        value_parser = enum_variants!(ReportingFormat),
        conflicts_with = "explain",
        conflicts_with = "list_rules",
        conflicts_with = "fix",
    )]
    pub reporting_format: ReportingFormat,

    /// Choose the failling threshold level for reported issues.
    #[arg(
        long,
        short = 'm',
        help = "Choose the failling threshold level for reported issues",
        default_value_t = Level::Error,
        value_parser = enum_variants!(Level),
        conflicts_with = "explain",
        conflicts_with = "list_rules",
        conflicts_with = "fix",
    )]
    pub minimum_level: Level,
}

impl LintCommand {
    pub const fn get_fix_classification(&self) -> SafetyClassification {
        if self.r#unsafe {
            SafetyClassification::Unsafe
        } else if self.potentially_unsafe {
            SafetyClassification::PotentiallyUnsafe
        } else {
            SafetyClassification::Safe
        }
    }
}

pub async fn execute(command: LintCommand, mut configuration: Configuration) -> Result<ExitCode, Error> {
    let interner = ThreadedInterner::new();

    // Determine the safety classification for the fixes.
    let fix_classification = command.get_fix_classification();

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

    let mut issues = if command.semantics_only {
        semantics_check(&interner, &source_manager, configuration.php_version).await?
    } else if command.compilation {
        compilation_check(&interner, &source_manager, configuration.php_version).await?
    } else {
        lint_check(&interner, &source_manager, &configuration).await?
    };

    if command.fix {
        let (changed, skipped_unsafe, skipped_potentially_unsafe) = fix_issues(
            &interner,
            &source_manager,
            issues,
            fix_classification,
            if command.format { Some((configuration.php_version, configuration.format.settings)) } else { None },
            command.dry_run,
        )
        .await?;

        if skipped_unsafe > 0 {
            tracing::warn!(
                "Skipped {} fixes because they were marked as unsafe. To apply those fixes, use the `--unsafe` flag.",
                skipped_unsafe
            );
        }

        if skipped_potentially_unsafe > 0 {
            tracing::warn!(
                "Skipped {} fixes because they were marked as potentially unsafe. To apply those fixes, use the `--potentially-unsafe` flag.",
                skipped_potentially_unsafe
            );
        }

        if changed == 0 {
            tracing::info!("No fixes were applied");

            return Ok(ExitCode::SUCCESS);
        }

        return Ok(if command.dry_run {
            tracing::info!("Found {} fixes that can be applied", changed);

            ExitCode::FAILURE
        } else {
            tracing::info!("Applied {} fixes successfully", changed);

            ExitCode::SUCCESS
        });
    }

    let issues_contain_errors = issues.has_minimum_level(command.minimum_level);

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

#[inline]
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
#[inline]
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

#[inline]
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
pub(super) async fn lint_check(
    interner: &ThreadedInterner,
    manager: &SourceManager,
    configuration: &Configuration,
) -> Result<IssueCollection, Error> {
    let php_version = configuration.php_version;
    let sources: Vec<_> = manager.source_ids_for_category(SourceCategory::UserDefined);
    let length = sources.len();

    let mut builder = ProjectBuilder::from_reflection(
        interner.clone(),
        reflect_non_user_sources(interner, php_version, manager).await?,
    );

    let scan_progress = create_progress_bar(length, "🔎  Scanning", ProgressBarTheme::Yellow);
    let mut handles = Vec::with_capacity(length);
    for source_id in sources {
        handles.push(tokio::spawn({
            let interner = interner.clone();
            let manager = manager.clone();
            let scan_progress = scan_progress.clone();

            async move {
                // Step 1: load the source
                let source = manager.load(&source_id)?;
                // Step 2: build module
                let module = Module::build(&interner, php_version, source, ModuleBuildOptions::default());

                scan_progress.inc(1);

                Result::<_, Error>::Ok(module)
            }
        }));
    }

    for handle in handles {
        builder.add_module(handle.await??);
    }

    remove_progress_bar(scan_progress);

    let Project { modules, mut reflection } = builder.build(true);
    let length = modules.len();
    let mut results = Vec::with_capacity(length + 1);
    results.push(reflection.take_issues());
    let linter = create_linter(interner, configuration, reflection);
    let lint_progress = create_progress_bar(length, "🧹  Linting", ProgressBarTheme::Red);
    let mut handles = Vec::with_capacity(length);
    for module in modules {
        handles.push(tokio::spawn({
            let linter = linter.clone();
            let lint_progress = lint_progress.clone();

            async move {
                let mut issues = linter.lint(&module);
                issues.extend(module.issues);
                if let Some(error) = &module.parse_error {
                    issues.push(Into::<Issue>::into(error));
                }

                lint_progress.inc(1);

                Result::<_, SourceError>::Ok(issues)
            }
        }));
    }

    for handle in handles {
        results.push(handle.await??);
    }

    remove_progress_bar(lint_progress);

    Ok(IssueCollection::from(results.into_iter().flatten()))
}

#[inline]
pub(super) async fn semantics_check(
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
                let module = Module::build(&interner, php_version, source, ModuleBuildOptions::validation());
                progress_bar.inc(1);

                Result::<_, Error>::Ok(module)
            }
        }));
    }

    let mut results = Vec::with_capacity(length);

    for handle in handles {
        let module = handle.await??;

        if let Some(error) = &module.parse_error {
            results.push(Into::<Issue>::into(error));
        }

        results.extend(module.issues);
    }

    remove_progress_bar(progress_bar);

    Ok(IssueCollection::from(results.into_iter()))
}

#[inline]
pub(super) async fn compilation_check(
    interner: &ThreadedInterner,
    manager: &SourceManager,
    php_version: PHPVersion,
) -> Result<IssueCollection, Error> {
    let sources: Vec<_> = manager.source_ids_for_category(SourceCategory::UserDefined);
    let length = sources.len();

    let mut project_builder = ProjectBuilder::from_reflection(
        interner.clone(),
        reflect_non_user_sources(interner, php_version, manager).await?,
    );

    let scan_progress = create_progress_bar(length, "🔎  Scanning", ProgressBarTheme::Yellow);
    let mut handles = Vec::with_capacity(length);
    for source_id in sources {
        handles.push(tokio::spawn({
            let interner = interner.clone();
            let manager = manager.clone();
            let scan_progress = scan_progress.clone();

            async move {
                // Step 1: load the source
                let source = manager.load(&source_id)?;
                // Step 2: build module
                let module = Module::build(&interner, php_version, source, ModuleBuildOptions::default());

                scan_progress.inc(1);

                Result::<_, Error>::Ok(module)
            }
        }));
    }

    let mut results = Vec::with_capacity(length);
    for handle in handles {
        let mut module = handle.await??;

        if let Some(error) = &module.parse_error {
            results.push(Into::<Issue>::into(error));
        }

        results.extend(std::mem::take(&mut module.issues));

        project_builder.add_module(module);
    }

    let Project { mut reflection, .. } = project_builder.build(true);
    results.extend(reflection.take_issues());

    remove_progress_bar(scan_progress);

    Ok(IssueCollection::from(results.into_iter()))
}

#[inline]
pub(super) async fn fix_issues(
    interner: &ThreadedInterner,
    source_manager: &SourceManager,
    issues: IssueCollection,
    fix_classification: SafetyClassification,
    format_settings: Option<(PHPVersion, FormatSettings)>,
    dry_run: bool,
) -> Result<(usize, usize, usize), Error> {
    let (plans, skipped_unsafe, skipped_potentially_unsafe) = filter_fix_plans(interner, issues, fix_classification);

    let total = plans.len();
    let progress_bar = create_progress_bar(total, "✨  Fixing", ProgressBarTheme::Cyan);
    let mut handles = Vec::with_capacity(total);
    for (source, plan) in plans.into_iter() {
        handles.push(tokio::spawn({
            let source_manager = source_manager.clone();
            let interner = interner.clone();
            let progress_bar = progress_bar.clone();

            async move {
                let source = source_manager.load(&source)?;
                let source_content = interner.lookup(&source.content);
                let mut new_content = plan.execute(source_content).get_fixed();
                if let Some((php_version, format_settings)) = format_settings {
                    let formatter = Formatter::new(&interner, php_version, format_settings);

                    new_content = match formatter.format_code(interner.lookup(&source.identifier.0), &new_content) {
                        Ok(content) => content,
                        Err(error) => {
                            tracing::error!("Failed to format the fixed code: {}", error);

                            new_content
                        }
                    };
                }

                let result = utils::apply_changes(&interner, &source_manager, &source, new_content, dry_run);

                progress_bar.inc(1);

                result
            }
        }));
    }

    let mut changed = 0;
    for handle in handles {
        if handle.await?? {
            changed += 1;
        }
    }

    remove_progress_bar(progress_bar);

    Ok((changed, skipped_unsafe, skipped_potentially_unsafe))
}

#[inline]
pub(super) fn filter_fix_plans(
    interner: &ThreadedInterner,
    issues: IssueCollection,
    classification: SafetyClassification,
) -> (Vec<(SourceIdentifier, FixPlan)>, usize, usize) {
    let mut skipped_unsafe = 0;
    let mut skipped_potentially_unsafe = 0;

    let mut results = vec![];
    for (source, plan) in issues.to_fix_plans() {
        if plan.is_empty() {
            continue;
        }

        let mut operations = vec![];
        for operation in plan.take_operations() {
            match operation.get_safety_classification() {
                SafetyClassification::Unsafe => {
                    if classification == SafetyClassification::Unsafe {
                        operations.push(operation);
                    } else {
                        skipped_unsafe += 1;

                        tracing::warn!(
                            "Skipping a fix for `{}` because it contains unsafe changes.",
                            interner.lookup(&source.0)
                        );
                    }
                }
                SafetyClassification::PotentiallyUnsafe => {
                    if classification == SafetyClassification::Unsafe
                        || classification == SafetyClassification::PotentiallyUnsafe
                    {
                        operations.push(operation);
                    } else {
                        skipped_potentially_unsafe += 1;

                        tracing::warn!(
                            "Skipping a fix for `{}` because it contains potentially unsafe changes.",
                            interner.lookup(&source.0)
                        );
                    }
                }
                SafetyClassification::Safe => {
                    operations.push(operation);
                }
            }
        }

        if !operations.is_empty() {
            results.push((source, FixPlan::from_operations(operations)));
        }
    }

    (results, skipped_unsafe, skipped_potentially_unsafe)
}
