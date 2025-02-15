use mago_interner::ThreadedInterner;
use mago_php_version::PHPVersion;
use mago_project::module::Module;
use mago_project::module::ModuleBuildOptions;
use mago_reflection::CodebaseReflection;
use mago_source::SourceCategory;
use mago_source::SourceManager;

use crate::error::Error;

/// Reflects on all non-user sources concurrently.
///
/// This function loads every source that is not user-defined (i.e., built-in or external)
/// from the provided `SourceManager`, parses them, and performs a structural reflection
/// using the `Module::reflect` method. The individual reflections are then merged into a single
/// [`CodebaseReflection`] instance.
///
/// # Arguments
///
/// * `interner` - A `ThreadedInterner` used for efficient symbol interning across threads.
/// * `php_version` - The PHP version guiding the parsing and reflection process.
/// * `manager` - A `SourceManager` that provides access to the external and built-in sources.
///
/// # Returns
///
/// A `Result` containing a combined [`CodebaseReflection`] that represents the aggregated
/// reflection of all non-user sources, or an [`Error`] if any source fails to load, parse, or reflect.
///
/// # Errors
///
/// This function returns an [`Error`] if:
/// - Any source cannot be loaded from the `SourceManager`.
/// - Parsing or reflecting on any source fails.
///
/// # Concurrency
///
/// Each non-user source is processed concurrently by spawning a Tokio task. The individual reflections
/// are then merged sequentially into a final `CodebaseReflection`.
pub async fn reflect_non_user_sources(
    interner: &ThreadedInterner,
    php_version: PHPVersion,
    manager: &SourceManager,
) -> Result<CodebaseReflection, Error> {
    // Collect all non-user source identifiers.
    let source_ids = manager.source_ids_except_category(SourceCategory::UserDefined);
    let total_sources = source_ids.len();

    // Spawn a task for each source to perform reflection concurrently.
    let mut reflection_tasks = Vec::with_capacity(total_sources);
    for source_id in source_ids {
        reflection_tasks.push(tokio::spawn({
            let manager = manager.clone();
            let interner = interner.clone();
            async move {
                let source = manager.load(&source_id)?;

                Ok::<CodebaseReflection, Error>(
                    Module::build(&interner, php_version, source, ModuleBuildOptions::reflection())
                        .reflection
                        .take()
                        .unwrap_or_default(),
                )
            }
        }));
    }

    // Merge all individual reflections into a unified reflection.
    let mut final_reflection = CodebaseReflection::new();
    for task in reflection_tasks {
        let source_reflection = task.await??; // Await task completion and propagate errors.
        final_reflection.merge(interner, source_reflection);
    }

    Ok(final_reflection)
}
