use ahash::HashSet;
use tokio::task::JoinHandle;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_interner::ThreadedInterner;
use mago_names::resolver::NameResolver;
use mago_source::Source;
use mago_source::SourceManager;
use mago_syntax::parser::parse_source;

use crate::error::Error;
use crate::utils::progress::ProgressBarTheme;
use crate::utils::progress::create_progress_bar;
use crate::utils::progress::remove_progress_bar;

/// Compiles `CodebaseMetadata` from all available sources concurrently with a progress bar.
///
/// Parses and scans each source file, updates the progress bar, and then populates
/// the codebase with resolved symbol information.
pub async fn compile_codebase_for_sources(
    manager: &SourceManager,
    symbol_references: &mut SymbolReferences,
    interner: &ThreadedInterner,
) -> Result<CodebaseMetadata, Error> {
    let source_ids = manager.source_ids();
    let mut compiled_codebase = CodebaseMetadata::new();

    let total_files = source_ids.len();
    let progress_bar = create_progress_bar(total_files, "Compiling", ProgressBarTheme::Magenta);

    let mut compilation_tasks: Vec<JoinHandle<Result<CodebaseMetadata, Error>>> = Vec::with_capacity(total_files);

    for source_id in source_ids {
        let interner_clone = interner.clone();
        let manager_clone = manager.clone();
        let progress_bar_clone = progress_bar.clone();

        compilation_tasks.push(tokio::spawn(async move {
            let source_file = manager_clone.load(&source_id)?;
            let source_metadata = extract_metadata_from_source(&source_file, &interner_clone);
            progress_bar_clone.inc(1);

            Ok(source_metadata)
        }));
    }

    for task_handle in compilation_tasks {
        let source_metadata = task_handle.await??;

        compiled_codebase.extend(source_metadata);
    }

    remove_progress_bar(progress_bar);

    tracing::trace!("Populating codebase with symbol references...");
    populate_codebase(&mut compiled_codebase, interner, symbol_references, HashSet::default(), HashSet::default());
    tracing::trace!("Codebase population complete.");

    Ok(compiled_codebase)
}

/// Extracts `CodebaseMetadata` from a single PHP source file.
fn extract_metadata_from_source(source: &Source, interner: &ThreadedInterner) -> CodebaseMetadata {
    let (program, parse_issues) = parse_source(interner, source);
    if parse_issues.is_some() {
        tracing::warn!(
            "Encountered parsing issue in {} during codebase compilation. Analysis may be incomplete.",
            interner.lookup(&source.identifier.0)
        );
    }

    let resolver = NameResolver::new(interner);
    let resolved_names = resolver.resolve(&program);

    scan_program(interner, source, &program, &resolved_names)
}
