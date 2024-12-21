use diffy::PatchFormatter;

use mago_interner::ThreadedInterner;
use mago_source::Source;
use mago_source::SourceManager;

use crate::error::Error;

/// Applies changes to the source file.
///
/// If `dry_run` is `true`, it compares the original and modified content,
/// displays a diff with context around changes, and does not write to disk.
///
/// If `dry_run` is `false`, it writes the formatted content to the source manager.
///
/// # Arguments
///
/// * `interner` - Reference to the `ThreadedInterner`.
/// * `source_manager` - Reference to the `SourceManager`.
/// * `source` - Reference to the `Source` being processed.
/// * `changed_code` - The formatted content as a `String`.
/// * `dry_run` - Boolean flag indicating whether to perform a dry run.
///
/// # Returns
///
/// * `Result<bool, SourceError>` - A result indicating whether the source was changed.
pub fn apply_changes(
    interner: &ThreadedInterner,
    source_manager: &SourceManager,
    source: &Source,
    changed_code: String,
    dry_run: bool,
) -> Result<bool, Error> {
    let original_content = interner.lookup(&source.content);
    if original_content == changed_code {
        return Ok(false);
    }

    if dry_run {
        let source_name = interner.lookup(&source.identifier.0);
        let patch = diffy::create_patch(original_content, changed_code.as_str());

        mago_feedback::progress::GLOBAL_PROGRESS_MANAGER.suspend(|| {
            let formatter = PatchFormatter::new().with_color();

            println!("diff of '{}':", source_name);
            println!("{}", formatter.fmt_patch(&patch));
        });
    } else {
        source_manager.write(source.identifier, changed_code)?;
    }

    Ok(true)
}
