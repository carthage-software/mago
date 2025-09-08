use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use serde::Deserialize;
use serde::Serialize;

use crate::change::Change;
use crate::change::ChangeLog;
use crate::error::DatabaseError;
use crate::file::File;
use crate::file::FileId;
use crate::file::FileType;
use crate::file::line_starts;
use crate::operation::FilesystemOperation;

mod utils;

pub mod change;
pub mod error;
pub mod exclusion;
pub mod file;
pub mod loader;

mod operation;

/// A mutable database for managing a collection of project files.
///
/// This struct acts as the primary "builder" for your file set. It is optimized
/// for efficient additions, updates, and deletions. Once you have loaded all
/// files and performed any initial modifications, you can create a high-performance,
/// immutable snapshot for fast querying by calling [`read_only`](Self::read_only).
///
/// While this structure implements [`Clone`](std::clone::Clone), it is not intended
/// for frequent cloning. Instead, it is designed to be used as a single mutable
/// instance that you modify in place. Cloning is provided for scenarios where
/// you need to create a backup or checkpoint of the current state before making
/// further changes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Database {
    /// Maps a file's logical name to its `File` object for fast name-based access.
    files: HashMap<Cow<'static, str>, Arc<File>>,
    /// Maps a file's stable ID back to its logical name for fast ID-based mutations.
    id_to_name: HashMap<FileId, Cow<'static, str>>,
}

/// An immutable, read-optimized snapshot of a file database.
///
/// This structure is designed for high-performance lookups and iteration. It stores
/// all files in a contiguous, sorted vector and uses multiple `HashMap` indices
/// to provide $O(1)$ average-time access to files by their ID, name, or path.
///
/// A `ReadDatabase` is created via [`Database::read_only`].
#[derive(Debug)]
pub struct ReadDatabase {
    /// A contiguous list of all files, sorted by `FileId` for deterministic iteration.
    files: Vec<Arc<File>>,
    /// Maps a file's stable ID to its index in the `files` vector.
    id_to_index: HashMap<FileId, usize>,
    /// Maps a file's logical name to its index in the `files` vector.
    name_to_index: HashMap<Cow<'static, str>, usize>,
    /// Maps a file's absolute path to its index in the `files` vector.
    path_to_index: HashMap<PathBuf, usize>,
}

impl Database {
    /// Creates a new, empty `Database`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `Database` containing only a single file.
    ///
    /// This is a convenience constructor for situations, such as testing or
    /// single-file tools, where an operation requires a [`Database`]
    /// implementation but only needs to be aware of one file.
    pub fn single(file: File) -> Self {
        let mut db = Self::new();
        db.add(file);
        db
    }

    /// Adds a file to the database, overwriting any existing file with the same name.
    pub fn add(&mut self, file: File) -> FileId {
        let name = file.name.clone();
        let id = file.id;

        if let Some(old_file) = self.files.insert(name.clone(), Arc::new(file)) {
            self.id_to_name.remove(&old_file.id);
        }

        self.id_to_name.insert(id, name);

        id
    }

    /// Updates a file's content in-place using its stable `FileId`.
    ///
    /// This recalculates derived data like file size, line endings, and `FileRevision`.
    /// Returns `true` if a file with the given ID was found and updated.
    pub fn update(&mut self, id: FileId, new_contents: Cow<'static, str>) -> bool {
        if let Some(name) = self.id_to_name.get(&id)
            && let Some(file) = self.files.get_mut(name)
            && let Some(file) = Arc::get_mut(file)
        {
            file.contents = new_contents;
            file.size = file.contents.len() as u32;
            file.lines = line_starts(file.contents.as_ref()).collect();
            return true;
        }
        false
    }

    /// Deletes a file from the database using its stable `FileId`.
    ///
    /// Returns `true` if a file with the given ID was found and removed.
    pub fn delete(&mut self, id: FileId) -> bool {
        if let Some(name) = self.id_to_name.remove(&id) { self.files.remove(&name).is_some() } else { false }
    }

    /// Commits a [`ChangeLog`], applying all its recorded operations to the database
    /// and optionally writing them to the filesystem.
    ///
    /// # Arguments
    ///
    /// * `change_log`: The log of changes to apply.
    /// * `write_to_disk`: If `true`, changes for files that have a filesystem
    ///   path will be written to disk in parallel.
    ///
    /// # Errors
    ///
    /// Returns a [`DatabaseError`] if the log cannot be consumed or if any
    /// filesystem operation fails.
    pub fn commit(&mut self, change_log: ChangeLog, write_to_disk: bool) -> Result<(), DatabaseError> {
        let changes = change_log.into_inner()?;
        let mut fs_operations = if write_to_disk { Vec::new() } else { Vec::with_capacity(0) };

        for change in changes {
            match change {
                Change::Add(file) => {
                    if write_to_disk && let Some(path) = &file.path {
                        fs_operations.push(FilesystemOperation::Write(path.clone(), file.contents.clone()));
                    }

                    self.add(file);
                }
                Change::Update(id, contents) => {
                    if write_to_disk
                        && let Ok(file) = self.get(&id)
                        && let Some(path) = &file.path
                    {
                        fs_operations.push(FilesystemOperation::Write(path.clone(), contents.clone()));
                    }

                    self.update(id, contents);
                }
                Change::Delete(id) => {
                    if write_to_disk
                        && let Ok(file) = self.get(&id)
                        && let Some(path) = &file.path
                    {
                        fs_operations.push(FilesystemOperation::Delete(path.clone()));
                    }

                    self.delete(id);
                }
            }
        }

        // If requested, perform all collected filesystem operations in parallel.
        if write_to_disk {
            fs_operations.into_par_iter().try_for_each(|op| -> Result<(), DatabaseError> { op.execute() })?;
        }

        Ok(())
    }

    /// Creates an independent, immutable snapshot of the database.
    ///
    /// This is a potentially expensive one-time operation as it **clones** all file
    /// data. The resulting [`ReadDatabase`] is highly optimized for fast reads and
    /// guarantees a deterministic iteration order. The original `Database` is not
    /// consumed and can continue to be used.
    pub fn read_only(&self) -> ReadDatabase {
        let mut files_vec: Vec<Arc<File>> = self.files.values().cloned().collect();
        files_vec.sort_unstable_by_key(|f| f.id);

        let mut id_to_index = HashMap::with_capacity(files_vec.len());
        let mut name_to_index = HashMap::with_capacity(files_vec.len());
        let mut path_to_index = HashMap::with_capacity(files_vec.len());

        for (index, file) in files_vec.iter().enumerate() {
            id_to_index.insert(file.id, index);
            name_to_index.insert(file.name.clone(), index);
            if let Some(path) = &file.path {
                path_to_index.insert(path.clone(), index);
            }
        }

        ReadDatabase { files: files_vec, id_to_index, name_to_index, path_to_index }
    }
}

impl ReadDatabase {
    /// Creates a new `ReadDatabase` containing only a single file.
    ///
    /// This is a convenience constructor for situations, such as testing or
    /// single-file tools, where an operation requires a [`DatabaseReader`]
    /// implementation but only needs to be aware of one file.
    ///
    /// # Arguments
    ///
    /// * `file`: The single `File` to include in the database.
    pub fn single(file: File) -> Self {
        let mut id_to_index = HashMap::with_capacity(1);
        let mut name_to_index = HashMap::with_capacity(1);
        let mut path_to_index = HashMap::with_capacity(1);

        // The index for the single file will always be 0.
        id_to_index.insert(file.id, 0);
        name_to_index.insert(file.name.clone(), 0);
        if let Some(path) = &file.path {
            path_to_index.insert(path.clone(), 0);
        }

        Self { files: vec![Arc::new(file)], id_to_index, name_to_index, path_to_index }
    }
}

/// A universal interface for reading data from any database implementation.
///
/// This trait provides a common API for querying file data, abstracting over
/// whether the underlying source is the mutable [`Database`] or the read-optimized
/// [`ReadDatabase`]. This allows for writing generic code that can operate on either.
pub trait DatabaseReader {
    /// Retrieves a file's stable ID using its logical name.
    fn get_id(&self, name: &str) -> Option<FileId>;

    /// Retrieves a reference to a file using its stable `FileId`.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::FileNotFound` if no file with the given ID exists.
    fn get(&self, id: &FileId) -> Result<Arc<File>, DatabaseError>;

    /// Retrieves a reference to a file using its stable `FileId`.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::FileNotFound` if no file with the given ID exists.
    fn get_ref(&self, id: &FileId) -> Result<&File, DatabaseError>;

    /// Retrieves a reference to a file using its logical name.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::FileNotFound` if no file with the given name exists.
    fn get_by_name(&self, name: &str) -> Result<Arc<File>, DatabaseError>;

    /// Retrieves a reference to a file by its absolute filesystem path.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::FileNotFound` if no file with the given path exists.
    fn get_by_path(&self, path: &Path) -> Result<Arc<File>, DatabaseError>;

    /// Returns an iterator over all files in the database.
    ///
    /// The order is not guaranteed for `Database`, but is sorted by `FileId`
    /// for `ReadDatabase`, providing deterministic iteration.
    fn files(&self) -> impl Iterator<Item = Arc<File>>;

    /// Returns an iterator over all files of a specific `FileType`.
    fn files_with_type(&self, file_type: FileType) -> impl Iterator<Item = Arc<File>> {
        self.files().filter(move |file| file.file_type == file_type)
    }

    /// Returns an iterator over all files that do not match a specific `FileType`.
    fn files_without_type(&self, file_type: FileType) -> impl Iterator<Item = Arc<File>> {
        self.files().filter(move |file| file.file_type != file_type)
    }

    /// Returns an iterator over the stable IDs of all files in the database.
    fn file_ids(&self) -> impl Iterator<Item = FileId> {
        self.files().map(|file| file.id)
    }

    /// Returns an iterator over the stable IDs of all files of a specific `FileType`.
    fn file_ids_with_type(&self, file_type: FileType) -> impl Iterator<Item = FileId> {
        self.files_with_type(file_type).map(|file| file.id)
    }

    /// Returns an iterator over the stable IDs of all files that do not match a specific `FileType`.
    fn file_ids_without_type(&self, file_type: FileType) -> impl Iterator<Item = FileId> {
        self.files_without_type(file_type).map(|file| file.id)
    }

    /// Returns the total number of files in the database.
    fn len(&self) -> usize;

    /// Returns `true` if the database contains no files.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl DatabaseReader for Database {
    fn get_id(&self, name: &str) -> Option<FileId> {
        self.files.get(name).map(|f| f.id)
    }

    fn get(&self, id: &FileId) -> Result<Arc<File>, DatabaseError> {
        let name = self.id_to_name.get(id).ok_or(DatabaseError::FileNotFound)?;
        let file = self.files.get(name).ok_or(DatabaseError::FileNotFound)?;

        Ok(file.clone())
    }

    fn get_ref(&self, id: &FileId) -> Result<&File, DatabaseError> {
        let name = self.id_to_name.get(id).ok_or(DatabaseError::FileNotFound)?;
        self.files.get(name).map(|file| file.as_ref()).ok_or(DatabaseError::FileNotFound)
    }

    fn get_by_name(&self, name: &str) -> Result<Arc<File>, DatabaseError> {
        self.files.get(name).cloned().ok_or(DatabaseError::FileNotFound)
    }

    fn get_by_path(&self, path: &Path) -> Result<Arc<File>, DatabaseError> {
        self.files.values().find(|file| file.path.as_deref() == Some(path)).cloned().ok_or(DatabaseError::FileNotFound)
    }

    fn files(&self) -> impl Iterator<Item = Arc<File>> {
        self.files.values().cloned()
    }

    fn len(&self) -> usize {
        self.files.len()
    }
}

impl DatabaseReader for ReadDatabase {
    fn get_id(&self, name: &str) -> Option<FileId> {
        self.name_to_index.get(name).and_then(|&i| self.files.get(i)).map(|f| f.id)
    }

    fn get(&self, id: &FileId) -> Result<Arc<File>, DatabaseError> {
        let index = self.id_to_index.get(id).ok_or(DatabaseError::FileNotFound)?;

        self.files.get(*index).cloned().ok_or(DatabaseError::FileNotFound)
    }

    fn get_ref(&self, id: &FileId) -> Result<&File, DatabaseError> {
        let index = self.id_to_index.get(id).ok_or(DatabaseError::FileNotFound)?;

        self.files.get(*index).map(|file| file.as_ref()).ok_or(DatabaseError::FileNotFound)
    }

    fn get_by_name(&self, name: &str) -> Result<Arc<File>, DatabaseError> {
        self.name_to_index.get(name).and_then(|&i| self.files.get(i)).cloned().ok_or(DatabaseError::FileNotFound)
    }

    fn get_by_path(&self, path: &Path) -> Result<Arc<File>, DatabaseError> {
        self.path_to_index.get(path).and_then(|&i| self.files.get(i)).cloned().ok_or(DatabaseError::FileNotFound)
    }

    fn files(&self) -> impl Iterator<Item = Arc<File>> {
        self.files.iter().cloned()
    }

    fn len(&self) -> usize {
        self.files.len()
    }
}
