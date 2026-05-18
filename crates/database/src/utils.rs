use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::read;
#[cfg(not(windows))]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;

use crate::error::DatabaseError;
use crate::file::File;
use crate::file::FileType;

/// Borrows `bytes` as an [`OsStr`] for platform-native filesystem APIs.
///
/// On Unix, paths are arbitrary byte sequences; the borrow is direct. On Windows
/// (and other non-Unix platforms), paths are UTF-8 — invalid sequences fall back to
/// lossy decoding with replacement characters.
pub(crate) fn bytes_to_os_str(bytes: &[u8]) -> Cow<'_, OsStr> {
    #[cfg(not(windows))]
    {
        Cow::Borrowed(OsStr::from_bytes(bytes))
    }
    #[cfg(windows)]
    {
        match std::str::from_utf8(bytes) {
            Ok(s) => Cow::Borrowed(OsStr::new(s)),
            Err(_) => Cow::Owned(String::from_utf8_lossy(bytes).into_owned().into()),
        }
    }
}

/// Borrows `bytes` as a [`Path`].
pub(crate) fn bytes_to_path(bytes: &[u8]) -> Cow<'_, Path> {
    match bytes_to_os_str(bytes) {
        Cow::Borrowed(s) => Cow::Borrowed(Path::new(s)),
        Cow::Owned(s) => Cow::Owned(PathBuf::from(s)),
    }
}

/// Returns `bytes` as a UTF-8 string, replacing invalid sequences.
#[inline]
pub(crate) fn bytes_to_string_lossy(bytes: &[u8]) -> Cow<'_, str> {
    String::from_utf8_lossy(bytes)
}

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
    let bytes = read(path)?;

    if bytes.len() > MAXIMUM_FILE_SIZE {
        return Err(DatabaseError::FileTooLarge(path.to_path_buf(), bytes.len(), MAXIMUM_FILE_SIZE));
    }

    // Normalize to forward slashes for cross-platform determinism
    #[cfg(windows)]
    let logical_name = path
        .strip_prefix(workspace)
        .unwrap_or(path)
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .map(|i| if *i == b'\\' { b'/' } else { *i })
        .collect::<Vec<_>>();
    #[cfg(not(windows))]
    let logical_name = path.strip_prefix(workspace).unwrap_or(path).as_os_str().as_encoded_bytes().to_owned();

    Ok(File::new(Cow::Owned(logical_name), file_type, Some(path.to_path_buf()), Cow::Owned(bytes)))
}
