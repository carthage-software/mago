use std::borrow::Cow;
use std::sync::Arc;
use std::sync::Mutex;

use crate::error::DatabaseError;
use crate::file::File;
use crate::file::FileId;

/// Represents a single, deferred database operation.
///
/// An instruction to be applied to a `Database` as part of a [`ChangeLog`].
#[derive(Debug)]
pub enum Change {
    /// An instruction to add a new file.
    Add(File),
    /// An instruction to update an existing file, identified by its `FileId`.
    Update(FileId, Cow<'static, str>),
    /// An instruction to delete an existing file, identified by its `FileId`.
    Delete(FileId),
}

/// A thread-safe, cloneable transaction log for collecting database operations.
///
/// This struct acts as a "Unit of Work," allowing multiple threads to concurrently
/// record operations without directly mutating the `Database`. The collected changes
/// can then be applied later in a single batch operation. This pattern avoids lock
/// contention on the main database during processing.
#[derive(Clone, Debug)]
pub struct ChangeLog {
    pub(crate) changes: Arc<Mutex<Vec<Change>>>,
}

impl Default for ChangeLog {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeLog {
    /// Creates a new, empty `ChangeLog`.
    #[must_use]
    pub fn new() -> Self {
        Self { changes: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Records a request to add a new file.
    ///
    /// # Errors
    ///
    /// Returns a `DatabaseError::PoisonedLogMutex` if another thread panicked
    /// while holding the lock, leaving the change log in an unusable state.
    pub fn add(&self, file: File) -> Result<(), DatabaseError> {
        self.changes.lock().map_err(|_| DatabaseError::PoisonedLogMutex)?.push(Change::Add(file));
        Ok(())
    }

    /// Records a request to update an existing file's content by its `FileId`.
    ///
    /// # Errors
    ///
    /// Returns a `DatabaseError::PoisonedLogMutex` if another thread panicked
    /// while holding the lock, leaving the change log in an unusable state.
    pub fn update(&self, id: FileId, new_contents: Cow<'static, str>) -> Result<(), DatabaseError> {
        self.changes.lock().map_err(|_| DatabaseError::PoisonedLogMutex)?.push(Change::Update(id, new_contents));
        Ok(())
    }

    /// Records a request to delete a file by its `FileId`.
    ///
    /// # Errors
    ///
    /// Returns a `DatabaseError::PoisonedLogMutex` if another thread panicked
    /// while holding the lock, leaving the change log in an unusable state.
    pub fn delete(&self, id: FileId) -> Result<(), DatabaseError> {
        self.changes.lock().map_err(|_| DatabaseError::PoisonedLogMutex)?.push(Change::Delete(id));
        Ok(())
    }

    /// Returns a vector of all `FileId`s affected by the recorded changes.
    ///
    /// This includes IDs for files being added, updated, or deleted.
    ///
    /// # Errors
    ///
    /// - `DatabaseError::PoisonedLogMutex`: Returned if the internal lock was poisoned
    pub fn changed_file_ids(&self) -> Result<Vec<FileId>, DatabaseError> {
        let changes = self.changes.lock().map_err(|_| DatabaseError::PoisonedLogMutex)?;
        let mut ids = Vec::new();
        for change in &*changes {
            match change {
                Change::Add(file) => ids.push(file.id),
                Change::Update(id, _) => ids.push(*id),
                Change::Delete(id) => ids.push(*id),
            }
        }

        Ok(ids)
    }

    /// Consumes the change log and returns the vector of collected changes.
    ///
    /// This operation safely unwraps the underlying list of changes. It will
    /// only succeed if called on the last remaining reference to the change log,
    /// which guarantees that no other threads can be modifying the list.
    ///
    /// # Errors
    ///
    /// - `DatabaseError::ChangeLogInUse`: Returned if other `Arc` references to this change log still exist.
    /// - `DatabaseError::PoisonedLogMutex`: Returned if the internal lock was poisoned because another thread panicked while holding it.
    pub fn into_inner(self) -> Result<Vec<Change>, DatabaseError> {
        match Arc::try_unwrap(self.changes) {
            Ok(mutex) => mutex.into_inner().map_err(|_| DatabaseError::PoisonedLogMutex),
            Err(_) => Err(DatabaseError::ChangeLogInUse),
        }
    }
}
