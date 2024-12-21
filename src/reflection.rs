use crate::error::Error;
use mago_interner::ThreadedInterner;
use mago_names::Names;
use mago_parser::parse_source;
use mago_reflection::CodebaseReflection;
use mago_source::SourceIdentifier;
use mago_source::SourceManager;

/// Creates a reflection of all external sources managed by the `SourceManager`.
///
/// This function processes all external sources concurrently.
///
/// Each source is parsed, and reflected to generate a comprehensive `CodebaseReflection`.
///
/// # Arguments
///
/// - `interner`: A `ThreadedInterner` instance used for symbol interning across multiple threads.
/// - `manager`: A `SourceManager` that provides access to external sources.
///
/// # Returns
///
/// Returns a `CodebaseReflection` representing the combined reflection of all external sources.
///
/// # Errors
///
/// - Returns an `Error` if any source cannot be loaded, parsed, or reflected.
pub async fn reflect_all_external_sources(
    interner: &ThreadedInterner,
    manager: &SourceManager,
) -> Result<CodebaseReflection, Error> {
    // Collect all external source identifiers managed by the SourceManager.
    let source_ids = manager.external_source_ids().collect::<Vec<_>>();
    let total_sources = source_ids.len();

    // Create a vector to hold the async tasks for reflecting each source.
    let mut reflection_tasks = Vec::with_capacity(total_sources);
    for source_id in source_ids {
        reflection_tasks.push(tokio::spawn({
            let interner = interner.clone();
            let manager = manager.clone();

            async move { reflect_single_source(&interner, &manager, &source_id) }
        }));
    }

    // Combine individual reflections into a unified `CodebaseReflection`.
    let mut combined_reflection = CodebaseReflection::new();
    for task in reflection_tasks {
        let source_reflection = task.await??; // Await task completion and handle errors.
        combined_reflection = mago_reflector::merge(combined_reflection, source_reflection);
    }

    Ok(combined_reflection)
}

/// Reflects a single source into a `CodebaseReflection`.
///
/// This function loads the specified source, parses its content, resolves its names, and performs
/// reflection to generate a `CodebaseReflection` for the given source.
///
/// # Arguments
///
/// - `interner`: A `ThreadedInterner` instance used for symbol interning.
/// - `manager`: A `SourceManager` to load the source code.
/// - `source_id`: The identifier of the source to reflect.
///
/// # Returns
///
/// A `CodebaseReflection` representing the reflection of the single source.
///
/// # Errors
///
/// - Returns an `Error` if the source cannot be loaded.
fn reflect_single_source(
    interner: &ThreadedInterner,
    manager: &SourceManager,
    source_id: &SourceIdentifier,
) -> Result<CodebaseReflection, Error> {
    // Load the source code using the SourceManager.
    let source = manager.load(source_id)?;

    // Parse the source code into an intermediate representation (program).
    let (program, _) = parse_source(interner, &source);

    // Resolve names and symbols within the program.
    let names = Names::resolve(interner, &program);

    // Reflect the source into a `CodebaseReflection`.
    Ok(mago_reflector::reflect(interner, &source, &program, &names))
}
