//! Twig linting command implementation.
//!
//! This module implements the `mago twig:lint` command, which checks Twig templates
//! against a configurable set of linting rules. It works like `mago lint`, with its own
//! rules and its own `[twig.linter]` configuration.
//!
//! # Features
//!
//! - **Full Linting** (default): Run all enabled rules against the templates
//! - **Syntax Only** (`--semantics`): Only check that the templates parse
//! - **Pedantic Mode** (`--pedantic`): Enable every rule the configured Twig version supports
//! - **Targeted Linting** (`--only`): Run only specific rules by code
//!
//! # Rule Discovery
//!
//! - **List Rules** (`--list-rules`): Display all currently enabled rules
//! - **Explain Rule** (`--explain <CODE>`): Show detailed documentation for a specific rule

use std::path::PathBuf;
use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;

use mago_database::Database;
use mago_database::DatabaseConfiguration;
use mago_orchestrator::service::lint::LintMode;
use mago_orchestrator::service::twig_lint::TwigLintService;
use mago_reporting::IssueCollection;
use mago_twig_linter::registry::RuleRegistry;
use mago_twig_linter::rule::AnyRule;
use mago_twig_linter::rule_meta::RuleEntry;
use mago_twig_linter::settings::Settings;
use mago_twig_linter::version::TwigVersion;

use crate::commands::args::baseline_reporting::BaselineReportingArgs;
use crate::commands::args::substitution::SubstitutionArgs;
use crate::commands::lint::print_rule_explanation;
use crate::commands::lint::print_rule_not_found;
use crate::commands::lint::print_rule_table;
use crate::commands::outcome::CommandOutcome;
use crate::commands::stdin_input;
use crate::config::Configuration;
use crate::error::Error;
use crate::service::EditLanguage;
use crate::utils::create_orchestrator;
use crate::utils::drop_in_background;
use crate::utils::git;

/// Command for linting Twig templates.
///
/// # Examples
///
/// Lint all configured source paths:
/// ```text
/// mago twig:lint
/// ```
///
/// Lint specific files or directories:
/// ```text
/// mago twig:lint templates/
/// ```
///
/// Show documentation for a rule:
/// ```text
/// mago twig:lint --explain twig-no-dump
/// ```
#[derive(Parser, Debug, Default)]
#[command(
    name = "twig:lint",
    about = "Lints Twig templates for style, maintenance, and safety issues.",
    long_about = indoc::indoc! {"
        Checks Twig templates against a configurable set of rules, without parsing them as PHP.

        USAGE:

            mago twig:lint
            mago twig:lint templates/
            mago twig:lint --list-rules
            mago twig:lint --explain twig-no-dump
            mago twig:lint --only twig-no-dump,twig-duplicate-block

        By default, it lints the `.twig` files in the source paths defined in your `mago.toml`
        file. Files given explicitly are linted as Twig whatever their extension.
    "}
)]
pub struct TwigLintCommand {
    /// Specific files or directories to lint instead of using configuration.
    ///
    /// Files given here are linted as Twig regardless of their extension.
    #[arg()]
    pub path: Vec<PathBuf>,

    /// Skip linter rules and only check that templates parse.
    #[arg(long, short = 's', conflicts_with_all = ["list_rules", "explain", "only"])]
    pub semantics: bool,

    /// Enable every rule supported by the configured minimum Twig version.
    #[arg(long)]
    pub pedantic: bool,

    /// Show detailed documentation for a specific linter rule.
    ///
    /// Displays the rule's description and examples of good and bad code.
    /// Use the rule's code name, such as 'twig-no-dump'.
    #[arg(
        long,
        conflicts_with_all = ["list_rules", "sort", "stats", "fixable_only", "semantics", "reporting_target", "reporting_format"]
    )]
    pub explain: Option<String>,

    /// Show all currently enabled linter rules.
    ///
    /// Combine with --json for machine-readable output.
    #[arg(
        long,
        conflicts_with_all = ["explain", "sort", "stats", "fixable_only", "semantics", "reporting_target", "reporting_format"]
    )]
    pub list_rules: bool,

    /// Output rule information in JSON format.
    #[arg(long, requires = "list_rules")]
    pub json: bool,

    /// Run only specific rules, ignoring the configuration file.
    ///
    /// Provide a comma-separated list of rule codes. Syntax errors are still reported.
    #[arg(short, long, conflicts_with = "semantics", value_delimiter = ',')]
    pub only: Vec<String>,

    /// Only lint Twig files that are staged in git, and stage any fixes.
    ///
    /// Fails if not in a git repository.
    #[arg(long, conflicts_with_all = ["path", "list_rules", "explain", "substitutions"])]
    pub staged: bool,

    /// Read the file content from stdin and use the given path for baseline and reporting.
    ///
    /// Intended for editor integrations: pipe unsaved buffer content and pass the real file path.
    #[arg(long, conflicts_with_all = ["list_rules", "explain", "staged"])]
    pub stdin_input: bool,

    #[clap(flatten)]
    pub baseline_reporting: BaselineReportingArgs,

    /// File-content substitutions (`--substitute ORIG=TEMP`).
    #[clap(flatten)]
    pub substitution: SubstitutionArgs,
}

impl TwigLintCommand {
    /// Executes the Twig lint command.
    ///
    /// # Errors
    ///
    /// Returns an error if input, scanning, linting, fixing, or reporting fails.
    pub fn execute(self, mut configuration: Configuration, color_choice: ColorChoice) -> Result<CommandOutcome, Error> {
        if self.baseline_reporting.reporting.format_after_fix {
            return Err(Error::InvalidArgument(
                "twig:lint does not support --format-after-fix. Twig formatting must run as a separate step."
                    .to_owned(),
            ));
        }

        let editor_url = configuration.editor_url.take();
        let config = configuration.twig.as_ref().and_then(|twig| twig.linter.as_ref()).cloned().unwrap_or_default();
        let settings = config.settings(configuration.source.glob.to_database_settings());
        let only = (!self.only.is_empty()).then_some(self.only.as_slice());

        if let Some(explain_code) = self.explain {
            return explain_rule(&settings, &explain_code).map(CommandOutcome::from);
        }

        if self.list_rules {
            if !config.enabled {
                return list_rules(&[], self.json).map(CommandOutcome::from);
            }

            let registry = RuleRegistry::build(&settings, only, self.pedantic);

            return list_rules(registry.rules(), self.json).map(CommandOutcome::from);
        }

        let substitutions = self.substitution.resolve()?;
        let substitution_excludes: Vec<String> =
            substitutions.iter().map(|s| s.original.to_string_lossy().into_owned()).collect();

        let mut orchestrator = create_orchestrator(&configuration, color_choice, false, true, false);
        orchestrator.config.extensions = vec!["twig"];
        orchestrator.add_exclude_patterns(config.excludes.iter());
        orchestrator.add_exclude_patterns(substitution_excludes.iter());

        let mut processor = self.baseline_reporting.get_processor(
            color_choice,
            config.baseline.as_deref(),
            config.baseline_variant,
            editor_url,
            config.minimum_fail_level,
            self.staged || !self.path.is_empty() || self.stdin_input,
        );
        processor.issue_processor.edit_language = EditLanguage::Twig;

        if !config.enabled {
            tracing::info!("Twig linting is disabled.");

            let mut database = Database::new(DatabaseConfiguration::new(
                &configuration.source.workspace,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));
            let (exit_code, changed) =
                processor.process_issues(&orchestrator, &mut database, IssueCollection::new())?;

            return CommandOutcome::with_changes(exit_code, &database, changed);
        }

        let stdin_override = stdin_input::resolve_stdin_override(
            self.stdin_input,
            &self.path,
            &configuration.source.workspace,
            &mut orchestrator,
        )?;

        if !self.stdin_input && self.staged {
            let staged_paths: Vec<_> = git::get_staged_file_paths(&configuration.source.workspace)?
                .into_iter()
                .filter(|path| path.extension().is_some_and(|extension| extension == "twig"))
                .collect();
            if staged_paths.is_empty() {
                tracing::info!("No staged Twig files to lint.");
                return Ok(ExitCode::SUCCESS.into());
            }

            if self.baseline_reporting.reporting.fix {
                git::ensure_staged_files_are_clean(&configuration.source.workspace, &staged_paths)?;
            }

            orchestrator.set_source_paths(staged_paths.iter().map(|p| p.to_string_lossy().to_string()));
        } else if !self.stdin_input && !self.path.is_empty() {
            stdin_input::set_source_paths_from_paths(&mut orchestrator, &self.path);
        }

        for substitution in &substitutions {
            orchestrator.config.paths.push(substitution.temporary.to_string_lossy().into_owned());
        }

        let mut database = orchestrator.load_database(&configuration.source.workspace, false, None, stdin_override)?;
        let service = TwigLintService::new(database.read_only(), settings, orchestrator.config.use_progress_bars);
        let issues =
            service.lint(if self.semantics { LintMode::SemanticsOnly } else { LintMode::Full }, only, self.pedantic)?;

        let (exit_code, changed_file_ids) = processor.process_issues(&orchestrator, &mut database, issues)?;
        let outcome = CommandOutcome::with_changes(exit_code, &database, changed_file_ids.iter().copied())?;

        if self.staged && !changed_file_ids.is_empty() {
            git::stage_files(&configuration.source.workspace, &database, changed_file_ids)?;
        }

        drop_in_background(database.into_static());

        Ok(outcome)
    }
}

/// Displays detailed documentation for a specific Twig linting rule.
///
/// Every rule can be explained, including disabled ones and ones that need a newer
/// Twig version than the configured minimum.
///
/// # Returns
///
/// - `Ok(ExitCode::SUCCESS)` if the rule was found and explained
/// - `Ok(ExitCode::FAILURE)` if the rule code doesn't exist
pub fn explain_rule(settings: &Settings, code: &str) -> Result<ExitCode, Error> {
    let settings = Settings { version: TwigVersion::LATEST, ..settings.clone() };
    let registry = RuleRegistry::build(&settings, Some(&[code.to_owned()]), true);
    let Some(rule) = registry.rules().first() else {
        print_rule_not_found(code);

        return Ok(ExitCode::FAILURE);
    };

    let meta = rule.meta();
    print_rule_explanation(
        meta.name,
        meta.description,
        meta.code,
        meta.category.as_str(),
        meta.good_example,
        meta.bad_example,
        "mago twig:lint",
    );

    Ok(ExitCode::SUCCESS)
}

/// Lists the given Twig linting rules and their configured levels, as a table or as JSON.
///
/// # Returns
///
/// Always returns `Ok(ExitCode::SUCCESS)` since listing rules cannot fail.
pub fn list_rules(rules: &[AnyRule], json: bool) -> Result<ExitCode, Error> {
    if json {
        let entries: Vec<_> = rules.iter().map(|r| RuleEntry { meta: r.meta(), level: r.level() }).collect();

        println!("{}", serde_json::to_string_pretty(&entries)?);

        return Ok(ExitCode::SUCCESS);
    }

    let rows: Vec<_> = rules
        .iter()
        .map(|rule| {
            let meta = rule.meta();

            (meta.name, meta.code, rule.level(), meta.category.as_str())
        })
        .collect();

    print_rule_table(&rows, "mago twig:lint");

    Ok(ExitCode::SUCCESS)
}
