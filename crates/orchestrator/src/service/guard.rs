use std::sync::Arc;

use mago_codex::metadata::CodebaseMetadata;
use mago_database::ReadDatabase;
use mago_guard::ArchitecturalGuard;
use mago_guard::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_syntax::parser::parse_file;

use crate::error::OrchestratorError;
use crate::service::pipeline::StatelessParallelPipeline;
use crate::service::pipeline::StatelessReducer;

/// Service responsible for running the guard pipeline.
#[derive(Debug)]
pub struct GuardService {
    /// The read-only database containing source files to guard.
    database: ReadDatabase,

    /// A codebase metadata of builtin symbols.
    codebase: CodebaseMetadata,

    /// The guard settings to configure the linting process.
    settings: Settings,

    /// Whether to display progress bars during guarding.
    use_progress_bars: bool,
}

impl GuardService {
    /// Creates a new instance of the `GuardService`.
    ///
    /// # Arguments
    ///
    /// * `database` - The read-only database containing source files to guard.
    /// * `codebase` - A codebase metadata of builtin symbols.
    /// * `settings` - The guard settings to configure the guarding process.
    /// * `use_progress_bars` - Whether to display progress bars during guarding.
    ///
    /// # Returns
    ///
    /// A new `GuardService` instance.
    pub fn new(
        database: ReadDatabase,
        codebase: CodebaseMetadata,
        settings: Settings,
        use_progress_bars: bool,
    ) -> Self {
        Self { database, codebase, settings, use_progress_bars }
    }

    /// Runs the guard pipeline on the codebase.
    ///
    /// # Returns
    ///
    /// A `Result` containing the final, aggregated [`IssueCollection`] for the
    /// entire project, or an [`OrchestratorError`].
    pub fn run(self) -> Result<IssueCollection, OrchestratorError> {
        const GUARD_PROGRESS_PREFIX: &str = "🛡️  Guarding";

        let pipeline = StatelessParallelPipeline::new(
            GUARD_PROGRESS_PREFIX,
            self.database,
            (Arc::new(self.codebase), self.settings),
            Box::new(GuardResultReducer),
            self.use_progress_bars,
        );

        pipeline.run(|(codebase, guard_settings), arena, source_file| {
            let mut issues = IssueCollection::new();

            let (program, parsing_error) = parse_file(arena, &source_file);

            if let Some(parsing_error) = parsing_error {
                issues.push(Issue::from(&parsing_error));

                return Ok(issues);
            }

            let resolved_names = NameResolver::new(arena).resolve(program);
            let guard = ArchitecturalGuard::new(guard_settings);
            let report = guard.check(&codebase, program, &resolved_names);

            issues.extend(
                // Report as issues
                report.report_into_issues(arena, &source_file, program),
            );

            Ok(issues)
        })
    }
}

/// The "reduce" step for the guard pipeline.
///
/// This struct aggregates the `IssueCollection` from each parallel task into a single,
/// final `IssueCollection` for the entire project.
#[derive(Debug, Clone)]
struct GuardResultReducer;

impl StatelessReducer<IssueCollection, IssueCollection> for GuardResultReducer {
    fn reduce(&self, results: Vec<IssueCollection>) -> Result<IssueCollection, OrchestratorError> {
        let mut aggregated_issues = IssueCollection::new();

        for result in results {
            aggregated_issues.extend(result);
        }

        Ok(aggregated_issues)
    }
}
