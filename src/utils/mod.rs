use std::borrow::Cow;

use diffy::PatchFormatter;

use mago_database::change::ChangeLog;
use mago_database::file::File;

use crate::error::Error;

pub mod logger;
pub mod progress;
pub mod version;

/// Processes the result of a modifying a single file.
///
/// This function compares the original file content with the newly modified content.
/// If there's a difference, it either prints a colorized diff to the console (if in
/// `dry_run` mode) or records an update operation in the provided [`ChangeLog`].
///
/// # Arguments
///
/// * `change_log`: The log where file updates are recorded when not in dry-run mode.
/// * `file`: The original file, used for comparison and context.
/// * `modified_contents`: The newly modified content.
/// * `dry_run`: If `true`, a diff is printed to standard output; otherwise, the
///   change is recorded in the `change_log`.
///
/// # Returns
///
/// Returns `true` if the file content was changed, `false` otherwise.
pub fn apply_update(
    change_log: &ChangeLog,
    file: &File,
    modified_contents: &str,
    dry_run: bool,
    check: bool,
    use_colors: bool,
) -> Result<bool, Error> {
    if file.contents == modified_contents {
        return Ok(false);
    }

    if check {
        return Ok(true);
    }

    if dry_run {
        let patch = diffy::create_patch(&file.contents, modified_contents);
        let mut formatter = PatchFormatter::new();
        if use_colors {
            formatter = formatter.with_color();
        };

        println!("diff of '{}':", file.name);
        println!("{}", formatter.fmt_patch(&patch));
    } else {
        change_log.update(file.id, Cow::Owned(modified_contents.to_owned()))?;
    }

    Ok(true)
}
