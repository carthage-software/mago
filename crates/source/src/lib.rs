use std::path::PathBuf;
use std::sync::Arc;

use ahash::HashMap;
use parking_lot::RwLock;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

use crate::error::SourceError;

pub mod error;

/// Represents the category of the source for a PHP construct.
///
/// This enum categorizes the origin of a source, based on where it is are defined.
/// The categories are useful for distinguishing between user-written code, vendor-provided libraries,
/// and built-in PHP features.
///
/// # Variants
///
/// - `BuiltIn`: Represents a construct that is part of PHP's core or extension libraries.
/// - `External`: Represents a construct defined in a vendor-provided or third-party library.
/// - `UserDefined`: Represents a construct written by the user or part of the current project.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum SourceCategory {
    /// Represents a PHP construct that is part of the PHP core or extension libraries.
    BuiltIn,

    /// Represents a PHP construct defined in vendor-provided or third-party libraries.
    External,

    /// Represents a PHP construct written by the user or part of the current project.
    #[default]
    UserDefined,
}

/// A unique identifier for a source.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct SourceIdentifier(pub StringIdentifier, pub SourceCategory);

/// Represents a source file.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Source {
    pub identifier: SourceIdentifier,
    pub path: Option<PathBuf>,
    pub content: StringIdentifier,
    pub size: usize,
    pub lines: Vec<usize>,
}

/// Trait for items that have a source.
pub trait HasSource {
    fn source(&self) -> SourceIdentifier;
}

/// Internal structure to store source information before full loading.
#[derive(Debug)]
struct SourceEntry {
    /// The file path (if any).
    path: Option<PathBuf>,
    /// The content, if already loaded, plus its size and line-start positions.
    content: Option<(StringIdentifier, usize, Vec<usize>)>,
}

/// Internal container for our maps. We keep two maps:
///  - one from SourceIdentifier → SourceEntry
///  - an auxiliary index from interned name → SourceIdentifier
#[derive(Debug)]
struct SourceManagerInner {
    sources: HashMap<SourceIdentifier, SourceEntry>,
    sources_by_name: HashMap<StringIdentifier, SourceIdentifier>,
}

/// A manager for sources.
///
/// This version replaces DashMap with a single inner structure protected by a
/// high-performance `RwLock` (from the parking_lot crate) and uses AHashMap for speed.
#[derive(Clone, Debug)]
pub struct SourceManager {
    /// The interner used for source names and content.
    interner: ThreadedInterner,
    /// Inner maps protected by a lock.
    inner: Arc<RwLock<SourceManagerInner>>,
}

/// Methods for SourceCategory.
impl SourceCategory {
    #[inline(always)]
    pub const fn is_built_in(&self) -> bool {
        matches!(self, Self::BuiltIn)
    }

    #[inline(always)]
    pub const fn is_external(&self) -> bool {
        matches!(self, Self::External)
    }

    #[inline(always)]
    pub const fn is_user_defined(&self) -> bool {
        matches!(self, Self::UserDefined)
    }
}

/// Methods for SourceIdentifier.
impl SourceIdentifier {
    #[inline(always)]
    pub fn dummy() -> Self {
        Self(StringIdentifier::empty(), SourceCategory::UserDefined)
    }

    /// Returns the interned string identifier.
    #[inline(always)]
    pub const fn value(&self) -> StringIdentifier {
        self.0
    }

    /// Returns the source category.
    #[inline(always)]
    pub const fn category(&self) -> SourceCategory {
        self.1
    }
}
/// Methods for Source.
impl Source {
    /// Creates a [`Source`] from a single piece of `content` without needing
    /// a full [`SourceManager`].
    ///
    /// This is particularly useful for quick parsing or one-off analyses
    /// where you do not need to manage multiple sources.
    ///
    /// # Arguments
    ///
    /// * `interner` - A reference to a [`ThreadedInterner`] used to intern
    ///   the `content` and store string identifiers.
    /// * `name` - A logical identifier for this source, such as `"inline"`
    ///   or `"my_script.php"`.
    /// * `content` - The actual PHP (or other) code string.
    #[inline(always)]
    pub fn standalone(interner: &ThreadedInterner, name: &str, content: &str) -> Self {
        let lines: Vec<_> = line_starts(content).collect();
        let size = content.len();
        let content_id = interner.intern(content);

        Self {
            identifier: SourceIdentifier(interner.intern(name), SourceCategory::UserDefined),
            path: None,
            content: content_id,
            size,
            lines,
        }
    }

    /// Retrieve the line number for the given byte offset.
    ///
    /// # Parameters
    ///
    /// - `offset`: The byte offset to retrieve the line number for.
    ///
    /// # Returns
    ///
    /// The line number for the given byte offset (0-based index).
    #[inline(always)]
    pub fn line_number(&self, offset: usize) -> usize {
        self.lines.binary_search(&offset).unwrap_or_else(|next_line| next_line - 1)
    }

    /// Retrieve the byte offset for the start of the given line.
    ///
    /// # Parameters
    ///
    /// - `line`: The line number to retrieve the start offset for.
    ///
    /// # Returns
    ///
    /// The byte offset for the start of the given line (0-based index).
    pub fn get_line_start_offset(&self, line: usize) -> Option<usize> {
        self.lines.get(line).copied()
    }

    /// Retrieve the byte offset for the end of the given line.
    ///
    /// # Parameters
    ///
    /// - `line`: The line number to retrieve the end offset for.
    ///
    /// # Returns
    ///
    /// The byte offset for the end of the given line (0-based index).
    pub fn get_line_end_offset(&self, line: usize) -> Option<usize> {
        match self.lines.get(line + 1) {
            Some(&end) => Some(end - 1),
            None if line == self.lines.len() - 1 => Some(self.size),
            _ => None,
        }
    }

    /// Retrieve the column number for the given byte offset.
    ///
    /// # Parameters
    ///
    /// - `offset`: The byte offset to retrieve the column number for.
    ///
    /// # Returns
    ///
    /// The column number for the given byte offset (0-based index).
    #[inline(always)]
    pub fn column_number(&self, offset: usize) -> usize {
        let line_start = self.lines.binary_search(&offset).unwrap_or_else(|next_line| self.lines[next_line - 1]);

        offset - line_start
    }
}

impl SourceManager {
    /// Creates a new source manager.
    #[inline(always)]
    pub fn new(interner: ThreadedInterner) -> Self {
        Self {
            interner,
            inner: Arc::new(RwLock::new(SourceManagerInner {
                sources: HashMap::default(),
                sources_by_name: HashMap::default(),
            })),
        }
    }

    /// Inserts a source with the given name and file path.
    #[inline(always)]
    pub fn insert_path(&self, name: impl AsRef<str>, path: PathBuf, category: SourceCategory) -> SourceIdentifier {
        let name_str = name.as_ref();
        let name_id = self.interner.intern(name_str);
        let source_id = SourceIdentifier(name_id, category);

        {
            let inner = self.inner.read();
            if inner.sources.contains_key(&source_id) {
                return source_id;
            }
        }

        let mut inner = self.inner.write();
        // Double-check to avoid duplicate insertion.
        if inner.sources.contains_key(&source_id) {
            return source_id;
        }
        inner.sources.insert(source_id, SourceEntry { path: Some(path), content: None });
        inner.sources_by_name.insert(name_id, source_id);
        source_id
    }

    /// Inserts a source with the given name and content.
    #[inline(always)]
    pub fn insert_content(
        &self,
        name: impl AsRef<str>,
        content: impl AsRef<str>,
        category: SourceCategory,
    ) -> SourceIdentifier {
        let name_str = name.as_ref();
        let content_str = content.as_ref();
        let name_id = self.interner.intern(name_str);

        {
            let inner = self.inner.read();
            if let Some(&source_id) = inner.sources_by_name.get(&name_id) {
                return source_id;
            }
        }

        let lines: Vec<_> = line_starts(content_str).collect();
        let size = content_str.len();
        let content_id = self.interner.intern(content_str);
        let source_id = SourceIdentifier(name_id, category);

        let mut inner = self.inner.write();
        if let Some(&existing) = inner.sources_by_name.get(&name_id) {
            return existing;
        }
        inner.sources.insert(source_id, SourceEntry { path: None, content: Some((content_id, size, lines)) });
        inner.sources_by_name.insert(name_id, source_id);
        source_id
    }

    /// Returns whether the manager contains a source with the given identifier.
    #[inline(always)]
    pub fn contains(&self, source_id: &SourceIdentifier) -> bool {
        let inner = self.inner.read();
        inner.sources.contains_key(source_id)
    }

    /// Returns all source identifiers.
    #[inline(always)]
    pub fn source_ids(&self) -> Vec<SourceIdentifier> {
        let inner = self.inner.read();
        inner.sources.keys().cloned().collect()
    }

    /// Returns source identifiers for the given category.
    #[inline(always)]
    pub fn source_ids_for_category(&self, category: SourceCategory) -> Vec<SourceIdentifier> {
        let inner = self.inner.read();
        inner.sources.keys().filter(|id| id.category() == category).cloned().collect()
    }

    /// Returns source identifiers for categories other than the given one.
    #[inline(always)]
    pub fn source_ids_except_category(&self, category: SourceCategory) -> Vec<SourceIdentifier> {
        let inner = self.inner.read();
        inner.sources.keys().filter(|id| id.category() != category).cloned().collect()
    }

    /// Loads the source for the given identifier.
    ///
    /// If the source content is already loaded, it is returned immediately.
    /// Otherwise the file is read from disk, processed, and cached.
    #[inline(always)]
    pub fn load(&self, source_id: &SourceIdentifier) -> Result<Source, SourceError> {
        // First, try to read without locking for update.
        {
            let inner = self.inner.read();
            if let Some(entry) = inner.sources.get(source_id) {
                if let Some((content, size, ref lines)) = entry.content {
                    return Ok(Source {
                        identifier: *source_id,
                        path: entry.path.clone(),
                        content,
                        size,
                        lines: lines.clone(),
                    });
                }
            }
        }

        // Retrieve the file path (must exist if content is not loaded).
        let path = {
            let inner = self.inner.read();
            let entry = inner.sources.get(source_id).ok_or(SourceError::UnavailableSource(*source_id))?;

            entry.path.clone().ok_or(SourceError::UnavailableSource(*source_id))?
        };

        // Perform file I/O outside the lock.
        let bytes = std::fs::read(&path).map_err(SourceError::IOError)?;
        let content_str = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(err) => {
                let s = err.into_bytes();
                let s = String::from_utf8_lossy(&s).into_owned();
                tracing::warn!("Source '{}' contains invalid UTF-8 sequence; behavior is undefined.", path.display());
                s
            }
        };
        let lines: Vec<_> = line_starts(&content_str).collect();
        let size = content_str.len();
        let content_id = self.interner.intern(&content_str);

        // Update the entry under a write lock.
        {
            let mut inner = self.inner.write();
            if let Some(entry) = inner.sources.get_mut(source_id) {
                // Check again in case another thread updated it meanwhile.
                if entry.content.is_none() {
                    entry.content = Some((content_id, size, lines.clone()));
                }
                Ok(Source { identifier: *source_id, path: entry.path.clone(), content: content_id, size, lines })
            } else {
                Err(SourceError::UnavailableSource(*source_id))
            }
        }
    }

    /// Writes updated content for the source with the given identifier.
    #[inline(always)]
    pub fn write(&self, source_id: SourceIdentifier, new_content: impl AsRef<str>) -> Result<(), SourceError> {
        let new_content_str = new_content.as_ref();
        let new_content_id = self.interner.intern(new_content_str);
        let new_lines: Vec<_> = line_starts(new_content_str).collect();
        let new_size = new_content_str.len();

        let path_opt = {
            let mut inner = self.inner.write();
            let entry = inner.sources.get_mut(&source_id).ok_or(SourceError::UnavailableSource(source_id))?;
            if let Some((old_content, _, _)) = entry.content {
                if old_content == new_content_id {
                    return Ok(());
                }
            }
            entry.content = Some((new_content_id, new_size, new_lines));
            entry.path.clone()
        };

        // If the source has an associated file, update it on disk.
        if let Some(ref path) = path_opt {
            std::fs::write(path, self.interner.lookup(&new_content_id)).map_err(SourceError::IOError)?;
        }

        Ok(())
    }

    /// Returns the number of sources.
    #[inline(always)]
    pub fn len(&self) -> usize {
        let inner = self.inner.read();
        inner.sources.len()
    }

    /// Returns true if there are no sources.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.read();
        inner.sources.is_empty()
    }
}

impl<T: HasSource> HasSource for Box<T> {
    #[inline(always)]
    fn source(&self) -> SourceIdentifier {
        self.as_ref().source()
    }
}

/// Returns an iterator over the starting byte offsets of each line in `source`.
#[inline(always)]
fn line_starts(source: &str) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(0).chain(memchr::memchr_iter(b'\n', source.as_bytes()).map(|i| i + 1))
}
