use std::borrow::Cow;
use std::path::Path;

use crate::error::DatabaseError;
use crate::file::File;
use crate::file::FileType;

const MAXIMUM_FILE_SIZE: usize = 1024 * 1024 * 1024;

/// Reads a file from disk and constructs a `File` object.
///
/// This function handles determining the file's logical name relative to the workspace,
/// reading its contents as bytes, and robustly converting those bytes to a string.
/// If the file contains invalid UTF-8 sequences, a warning is logged, and the
/// conversion is performed lossily, replacing invalid characters.
///
/// # Arguments
///
/// * `workspace`: The root directory of the project, used to calculate the logical name.
/// * `path`: The absolute path to the file to read.
/// * `file_type`: The [`FileType`] to assign to the created file.
///
/// # Errors
///
/// Returns a [`DatabaseError::IOError`] if the file cannot be read from the filesystem.
pub(crate) fn read_file(workspace: &Path, path: &Path, file_type: FileType) -> Result<File, DatabaseError> {
    let logical_name = path.strip_prefix(workspace).unwrap_or(path).to_string_lossy().to_string();

    let bytes = std::fs::read(path)?;
    if bytes.len() > MAXIMUM_FILE_SIZE {
        return Err(DatabaseError::FileTooLarge(path.to_path_buf(), bytes.len(), MAXIMUM_FILE_SIZE));
    }

    let contents = match str::from_utf8(&bytes) {
        Ok(s) => s.to_string(),
        Err(e) => {
            let warning_message = format!(
                "File `{}` contains invalid UTF-8 at byte {}. Lossy conversion applied, which may cause undefined behavior.",
                logical_name,
                e.valid_up_to()
            );

            match file_type {
                FileType::Host => tracing::warn!("{}", warning_message),
                FileType::Vendored | FileType::Builtin => tracing::info!("{}", warning_message),
            }

            String::from_utf8_lossy(&bytes).into_owned()
        }
    };

    Ok(File::new(Cow::Owned(logical_name), file_type, Some(path.to_path_buf()), Cow::Owned(contents)))
}
