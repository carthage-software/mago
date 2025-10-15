use mago_codex::metadata::CodebaseMetadata;
use mago_database::ReadDatabase;
use mago_guard::ArchitecturalGuard;
use mago_guard::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_syntax::parser::parse_file;

use crate::error::Error;
use crate::pipeline::StatelessParallelPipeline;
use crate::pipeline::StatelessReducer;

/// The "reduce" step for the guard pipeline.
///
/// This struct aggregates the `IssueCollection` from each parallel task into a single,
/// final `IssueCollection` for the entire project.
#[derive(Debug, Clone)]
struct GuardResultReducer;

impl StatelessReducer<IssueCollection, IssueCollection> for GuardResultReducer {
    fn reduce(&self, results: Vec<IssueCollection>) -> Result<IssueCollection, Error> {
        let mut aggregated_issues = IssueCollection::new();

        for result in results {
            aggregated_issues.extend(result);
        }

        Ok(aggregated_issues)
    }
}

/// Runs the parallel guard checking phase on a fully compiled codebase.
///
/// This function orchestrates the guard workflow using a [`ParallelPipeline`].
/// It takes a database containing all source files and the pre-compiled codebase
/// metadata, then runs architectural boundary checking on each "host" file in parallel.
///
/// # Arguments
///
/// * `database`: The read-only database containing all files to be checked.
/// * `codebase`: The pre-compiled `CodebaseMetadata` from the prelude.
/// * `symbol_references`: The pre-compiled `SymbolReferences` from the prelude.
/// * `guard_settings`: The configured settings for the guard.
///
/// # Returns
///
/// A `Result` containing the final, aggregated [`IssueCollection`] for the
/// entire project, or an [`Error`].
pub fn run_guard_pipeline(
    database: ReadDatabase,
    codebase: CodebaseMetadata,
    guard_settings: Settings,
) -> Result<IssueCollection, Error> {
    const GUARD_PROGRESS_PREFIX: &str = "🛡️  Guarding";

    let pipeline = StatelessParallelPipeline::new(
        GUARD_PROGRESS_PREFIX,
        database,
        (codebase, guard_settings),
        Box::new(GuardResultReducer),
        true,
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
