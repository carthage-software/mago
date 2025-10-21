use std::sync::Arc;

use mago_database::ReadDatabase;
use mago_linter::Linter;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

use crate::OrchestratorError;
use crate::service::pipeline::StatelessParallelPipeline;
use crate::service::pipeline::StatelessReducer;

/// Defines the different operational modes for the linter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LintMode {
    /// Runs only parsing and semantic checks. This is the fastest mode.
    SemanticsOnly,
    /// Runs all checks: semantics, compilation, and the full linter rule set.
    Full,
}

/// Service responsible for running the linting pipeline.
#[derive(Debug)]
pub struct LintService {
    /// The read-only database containing source files to lint.
    database: ReadDatabase,

    /// The linter settings to configure the linting process.
    settings: Settings,

    /// Whether to display progress bars during linting.
    use_progress_bars: bool,
}

impl LintService {
    /// Creates a new instance of the `LintService`.
    ///
    /// # Arguments
    ///
    /// * `database` - The read-only database containing source files to lint.
    /// * `settings` - The linter settings to configure the linting process.
    /// * `use_progress_bars` - Whether to display progress bars during linting.
    ///
    /// # Returns
    ///
    /// A new `LintService` instance.
    pub fn new(database: ReadDatabase, settings: Settings, use_progress_bars: bool) -> Self {
        Self { database, settings, use_progress_bars }
    }

    /// Creates a `RuleRegistry` based on the current settings.
    ///
    /// # Arguments
    ///
    /// * `only` - An optional list of specific rules to include.
    /// * `include_disabled` - Whether to include disabled rules in the registry.
    ///
    /// # Returns
    ///
    /// A configured `RuleRegistry` instance.
    pub fn create_registry(&self, only: Option<&[String]>, include_disabled: bool) -> RuleRegistry {
        RuleRegistry::build(&self.settings, only, include_disabled)
    }

    /// Runs the linting pipeline in the specified mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The operational mode for linting (semantics only or full).
    ///
    /// # Returns
    ///
    /// A `Result` containing the final `IssueCollection` or an `OrchestratorError`.
    pub fn lint(self, mode: LintMode) -> Result<IssueCollection, OrchestratorError> {
        const PROGRESS_BAR_THEME: &str = "🧹 Linting";

        let context = LintContext {
            php_version: self.settings.php_version,
            registry: Arc::new(self.create_registry(None, false)),
            mode,
        };

        let pipeline = StatelessParallelPipeline::new(
            PROGRESS_BAR_THEME,
            self.database,
            context,
            Box::new(LintResultReducer),
            self.use_progress_bars,
        );

        pipeline.run(|context, arena, file| {
            let (program, parsing_error) = parse_file(arena, &file);
            let resolved_names = NameResolver::new(arena).resolve(program);

            let mut issues = IssueCollection::new();
            if let Some(error) = parsing_error {
                issues.push(Issue::from(&error));
            }

            let semantics_checker = SemanticsChecker::new(context.php_version);
            issues.extend(semantics_checker.check(&file, program, &resolved_names));

            if context.mode == LintMode::Full {
                let linter = Linter::from_registry(arena, context.registry, context.php_version);

                issues.extend(linter.lint(&file, program, &resolved_names));
            }

            Ok(issues)
        })
    }
}

/// Shared, read-only context provided to each parallel linting task.
#[derive(Clone)]
struct LintContext {
    /// The target PHP version for analysis.
    pub php_version: PHPVersion,
    /// A pre-configured `RuleRegistry` instance.
    pub registry: Arc<RuleRegistry>,
    /// The operational mode, determining which checks to run.
    pub mode: LintMode,
}

/// The "reduce" step for the linting pipeline.
///
/// This struct implements both stateful and stateless reduction, aggregating
/// `IssueCollection`s from parallel tasks into a single, final collection.
#[derive(Debug)]
struct LintResultReducer;

impl StatelessReducer<IssueCollection, IssueCollection> for LintResultReducer {
    fn reduce(&self, results: Vec<IssueCollection>) -> Result<IssueCollection, OrchestratorError> {
        let mut final_issues = IssueCollection::new();
        for issues in results {
            final_issues.extend(issues);
        }

        Ok(final_issues)
    }
}
