use std::path::PathBuf;

use globset::Error as GlobSetError;

/// The primary error type for all database loading and mutation operations.
///
/// This enum consolidates errors from various sources, including file I/O,
/// pattern matching, and concurrent access issues, into a single, unified type.
#[derive(Debug)]
pub enum DatabaseError {
    /// An attempt was made to access a file that does not exist in the database.
    FileNotFound,
    /// An error occurred during a filesystem read or write operation.
    IOError(std::io::Error),
    /// The set of user-provided glob patterns could not be compiled into a `GlobSet`.
    InvalidGlobSet(GlobSetError),
    /// The file being loaded into the database is too large to be processed.
    FileTooLarge(PathBuf, usize, usize),
    /// An attempt was made to commit or consume a `ChangeLog` while other
    /// references to it still exist, indicating that other threads may not have
    /// finished their work.
    ChangeLogInUse,
    /// The lock on a `ChangeLog` was "poisoned."
    ///
    /// This happens when a thread panics while holding the lock, leaving the
    /// data in an unrecoverable and potentially inconsistent state.
    PoisonedLogMutex,
    /// Failed to initialize the file system watcher.
    WatcherInit(notify::Error),
    /// Failed to add a path to the file system watcher.
    WatcherWatch(notify::Error),
    /// Attempted to wait on a watcher that is not currently watching.
    WatcherNotActive,
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound => write!(f, "file not found in database"),
            Self::IOError(err) => write!(f, "I/O error: {err}"),
            Self::InvalidGlobSet(err) => write!(f, "failed to build exclusion filter from patterns: {err}"),
            Self::FileTooLarge(path, size, max_size) => {
                write!(
                    f,
                    "file at {} is too large to be processed: {} bytes (maximum is {} bytes)",
                    path.display(),
                    size,
                    max_size
                )
            }
            Self::ChangeLogInUse => {
                write!(f, "cannot commit changelog because it is still in use by another thread")
            }
            Self::PoisonedLogMutex => {
                write!(f, "changelog is in an unrecoverable state because a thread panicked while modifying it")
            }
            Self::WatcherInit(err) => write!(f, "failed to initialize file watcher: {err}"),
            Self::WatcherWatch(err) => write!(f, "failed to watch path: {err}"),
            Self::WatcherNotActive => write!(f, "watcher is not currently watching - call watch() first"),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IOError(err) => Some(err),
            Self::InvalidGlobSet(err) => Some(err),
            Self::WatcherInit(err) | Self::WatcherWatch(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DatabaseError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}

impl From<GlobSetError> for DatabaseError {
    fn from(error: GlobSetError) -> Self {
        Self::InvalidGlobSet(error)
    }
}
