use std::process::ExitCode;
use std::str::FromStr;

use clap::Parser;

use mago_interner::ThreadedInterner;
use mago_names::resolver::NameResolver;
use mago_reference::Reference;
use mago_reference::ReferenceFinder;
use mago_reference::ReferenceKind;
use mago_reference::query::Query;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::reporter::Reporter;
use mago_reporting::reporter::ReportingFormat;
use mago_reporting::reporter::ReportingTarget;
use mago_source::SourceCategory;
use mago_source::SourceManager;
use mago_syntax::parser::parse_source;

use crate::config::Configuration;
use crate::enum_variants;
use crate::error::Error;
use crate::source;
use crate::utils::progress::ProgressBarTheme;
use crate::utils::progress::create_progress_bar;
use crate::utils::progress::remove_progress_bar;

/// The `find` subcommand used to locate references to a symbol (or pattern) in the codebase.
///
/// # Overview
///
/// * Parses a **query** string into a [`Query`](mago_reference::query::Query),
///   which can represent exact matches, substring, etc.
/// * Optionally includes external sources (e.g., vendor or system files).
/// * Uses a [`ReferenceFinder`] to locate all matching references in parallel.
/// * Reports them at level `Info` via the chosen `ReportingTarget` and `ReportingFormat`.
#[derive(Parser, Debug)]
#[command(
    name = "find",
    about = "Find references to a symbol in the codebase",
    long_about = r#"
Searches for symbol references (imports, uses, definitions, etc.) in user-defined or external sources.

The query syntax supports:

  =Exact   : an exact match
  ^Prefix  : a starts-with match
   Substr  : a contains match
   Suffix$ : an ends-with match

A Query can be case-sensitive by prefixing with "c:" or case-insensitive with "i:",
If no prefix is provided, the query is case-insensitive by default.
"#
)]
pub struct FindCommand {
    /// The query string, e.g. "=MyClass", "^myFunc", "partialName", "someName$".
    #[arg(help = "The query specifying how references should match")]
    pub query: String,

    /// Whether to include external (vendor or system) files when searching for references.
    #[arg(short, long, help = "Include external sources (e.g., vendor) in the search")]
    pub include_external: bool,

    /// Specify where the results should be reported (e.g. CLI, JSON, file).
    #[arg(
        long,
        default_value_t,
        help = "Specify where the results should be reported",
        ignore_case = true,
        value_parser = enum_variants!(ReportingTarget)
    )]
    pub reporting_target: ReportingTarget,

    /// Choose the format (human-friendly, JSON, etc.) for reporting issues.
    #[arg(
        long,
        default_value_t,
        help = "Choose the format for reporting issues",
        ignore_case = true,
        value_parser = enum_variants!(ReportingFormat)
    )]
    pub reporting_format: ReportingFormat,
}

/// Executes the `find` command, returning an exit status code.
///
/// 1. Parses the user’s query into a [`Query`].
/// 2. Loads relevant sources (internal only or including external).
/// 3. Spawns tasks to find references across all sources in parallel.
/// 4. Reports the discovered references using [`Reporter`].
///
/// If the query string is invalid, returns `ExitCode::FAILURE`.
pub async fn execute(command: FindCommand, configuration: Configuration) -> Result<ExitCode, Error> {
    let interner = ThreadedInterner::new();

    // Attempt to parse the query from user input
    let query = match Query::from_str(&command.query) {
        Ok(query) => query,
        Err(_) => {
            tracing::error!("Failed to parse query: \"{}\"", command.query);
            eprintln!("Error: Invalid query syntax. Examples:\n  =Exact   ^Prefix   partial   suffix$");
            eprintln!("Optionally prefix with 'c:' for case-sensitive or 'i:' for case-insensitive.");
            return Ok(ExitCode::FAILURE);
        }
    };

    // Load the source files
    let source_manager = source::load(
        &interner,
        &configuration.source,
        command.include_external,
        /* ??? disable built-ins? */ false,
    )
    .await?;

    // Gather references
    let references = find_references(&interner, &source_manager, query, command.include_external).await?;

    // Convert references to issues, then report
    Reporter::new(interner.clone(), source_manager, command.reporting_target).report(
        references.into_iter().map(|reference| reference_to_issue(&interner, reference)).collect::<Vec<_>>(),
        command.reporting_format,
    )?;

    Ok(ExitCode::SUCCESS)
}

/// Finds references to a symbol or pattern across multiple source files.
///
/// # Parameters
///
/// - `interner`: The global string interner for symbol lookups.
/// - `manager`: The [`SourceManager`] for loading and iterating code.
/// - `query`: The parsed [`Query`] representing how to match references.
/// - `include_externals`: If `true`, includes external (vendor/system) sources.
///
/// # Returns
/// A list of all [`Reference`](mago_reference::Reference) matches discovered.
///
/// This function spawns parallel tasks (one per source file) to build a [`Program`]
/// and run the [`ReferenceFinder`].
pub async fn find_references(
    interner: &ThreadedInterner,
    manager: &SourceManager,
    query: Query,
    include_externals: bool,
) -> Result<Vec<Reference>, Error> {
    // Choose which sources to analyze
    let sources: Vec<_> = if include_externals {
        manager.source_ids_for_category(SourceCategory::UserDefined)
    } else {
        manager.source_ids_except_category(SourceCategory::BuiltIn)
    };

    let length = sources.len();
    let progress_bar = create_progress_bar(length, "🔎  Scanning", ProgressBarTheme::Yellow);

    // Spawn tasks to find references in each source
    let mut handles = Vec::with_capacity(length);
    for source_id in sources {
        let interner = interner.clone();
        let manager = manager.clone();
        let progress_bar = progress_bar.clone();
        let query = query.clone();

        handles.push(tokio::spawn(async move {
            let source = manager.load(&source_id)?;
            let program = parse_source(&interner, &source).0;
            let resolved_names = NameResolver::new(&interner).resolve(&program);
            let references = ReferenceFinder::new(&interner).find(&program, &resolved_names, query);

            progress_bar.inc(1);

            Result::<_, Error>::Ok(references)
        }));
    }

    // Collect all results
    let mut all_references = Vec::with_capacity(length);
    for handle in handles {
        all_references.extend(handle.await??);
    }

    remove_progress_bar(progress_bar);

    Ok(all_references)
}

/// Converts a discovered [`Reference`](mago_reference::Reference) into an [`Issue`],
/// suitable for CLI display at the `Note` level.
pub fn reference_to_issue(interner: &ThreadedInterner, reference: Reference) -> Issue {
    let symbol_name = interner.lookup(&reference.value);

    match reference.kind {
        ReferenceKind::Import => Issue::note(format!("Reference: imported symbol `{symbol_name}`")).with_annotation(
            Annotation::primary(reference.span).with_message(format!("This import references `{symbol_name}`.")),
        ),

        ReferenceKind::Usage => Issue::note(format!("Reference: used symbol `{symbol_name}`")).with_annotation(
            Annotation::primary(reference.span).with_message(format!("Symbol `{symbol_name}` is used here.")),
        ),

        ReferenceKind::Definition => Issue::note(format!("Reference: definition of `{symbol_name}`")).with_annotation(
            Annotation::primary(reference.span).with_message(format!("Symbol `{symbol_name}` is defined here.")),
        ),

        ReferenceKind::Implementation => Issue::note(format!("Reference: implementation of `{symbol_name}`"))
            .with_annotation(
                Annotation::primary(reference.span)
                    .with_message(format!("Symbol `{symbol_name}` is implemented here.")),
            ),

        ReferenceKind::Extension => Issue::note(format!("Reference: extension of `{symbol_name}`")).with_annotation(
            Annotation::primary(reference.span).with_message(format!("Class `{symbol_name}` is extended here.")),
        ),
    }
}
