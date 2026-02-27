use std::borrow::Cow;
use std::fs::File as StdFile;
use std::io::Read;
use std::path::Path;

use crate::error::DatabaseError;
use crate::file::File;
use crate::file::FileType;

/// The used hint for pre-allocating file content buffer.
///
/// File sizes larger than this will still be handled correctly,
/// but usually files are smaller than this.
const FILE_SIZE_HINT: usize = 12 * 1024;

/// The maximum allowed file size (256 MiB).
const MAXIMUM_FILE_SIZE: usize = 256 * 1024 * 1024;

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
    let mut bytes = Vec::with_capacity(FILE_SIZE_HINT);
    let mut file = StdFile::open(path)?;
    file.read_to_end(&mut bytes)?;

    if bytes.len() > MAXIMUM_FILE_SIZE {
        return Err(DatabaseError::FileTooLarge(path.to_path_buf(), bytes.len(), MAXIMUM_FILE_SIZE));
    }

    // Normalize to forward slashes for cross-platform determinism
    #[cfg(windows)]
    let logical_name = path.strip_prefix(workspace).unwrap_or(path).to_string_lossy().replace('\\', "/");
    #[cfg(not(windows))]
    let logical_name = path.strip_prefix(workspace).unwrap_or(path).to_string_lossy().into_owned();
    let contents = if simdutf8::basic::from_utf8(&bytes).is_ok() {
        unsafe {
            // SAFETY: We just validated it with simdutf8, no need to check again.
            String::from_utf8_unchecked(bytes)
        }
    } else {
        let warning_message = format!(
            "File `{logical_name}` contains invalid UTF-8. Lossy conversion applied, which may cause undefined behavior.",
        );

        match file_type {
            FileType::Host => tracing::warn!("{}", warning_message),
            FileType::Vendored | FileType::Builtin => tracing::info!("{}", warning_message),
        }

        String::from_utf8_lossy(&bytes).into_owned()
    };

    Ok(File::new(Cow::Owned(logical_name), file_type, Some(path.to_path_buf()), Cow::Owned(contents)))
}
