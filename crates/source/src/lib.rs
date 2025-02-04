use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
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
    /// The content, if already loaded, along with its size and line-starts.
    content: Option<(StringIdentifier, usize, Vec<usize>)>,
}

/// A manager for sources.
#[derive(Clone, Debug)]
pub struct SourceManager {
    /// The interner used for source names and content.
    interner: ThreadedInterner,
    /// Map of SourceIdentifier -> SourceEntry.
    sources: Arc<DashMap<SourceIdentifier, SourceEntry>>,
    /// Auxiliary index from interned name to SourceIdentifier.
    sources_by_name: Arc<DashMap<StringIdentifier, SourceIdentifier>>,
}

impl SourceIdentifier {
    #[inline(always)]
    pub fn dummy() -> Self {
        Self(StringIdentifier::empty(), SourceCategory::UserDefined)
    }

    /// Returns the string identifier of the source.
    #[inline(always)]
    pub const fn value(&self) -> StringIdentifier {
        self.0
    }

    #[inline(always)]
    pub const fn category(&self) -> SourceCategory {
        self.1
    }
}

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
        Self { interner, sources: Arc::new(DashMap::new()), sources_by_name: Arc::new(DashMap::new()) }
    }

    /// Inserts a source with the given name and path.
    #[inline(always)]
    pub fn insert_path(&self, name: impl AsRef<str>, path: PathBuf, category: SourceCategory) -> SourceIdentifier {
        let name_id = self.interner.intern(&name);
        let source_id = SourceIdentifier(name_id, category);
        // Fast-path: if already present, return.
        if self.sources.contains_key(&source_id) {
            return source_id;
        }

        self.sources.insert(source_id, SourceEntry { path: Some(path), content: None });
        self.sources_by_name.insert(name_id, source_id);
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
        let name_id = self.interner.intern(&name);
        // Try our auxiliary index first.
        if let Some(source_id) = self.sources_by_name.get(&name_id).map(|v| *v) {
            return source_id;
        }
        let source_id = SourceIdentifier(name_id, category);
        let lines: Vec<_> = line_starts(content.as_ref()).collect();
        let size = content.as_ref().len();
        let content_id = self.interner.intern(content);
        self.sources.insert(source_id, SourceEntry { path: None, content: Some((content_id, size, lines)) });
        self.sources_by_name.insert(name_id, source_id);
        source_id
    }

    /// Returns whether the manager contains a source with the given identifier.
    #[inline(always)]
    pub fn contains(&self, source_id: &SourceIdentifier) -> bool {
        self.sources.contains_key(source_id)
    }

    /// Returns an iterator over all source identifiers.
    #[inline(always)]
    pub fn source_ids(&self) -> impl Iterator<Item = SourceIdentifier> + '_ {
        self.sources.iter().map(|entry| *entry.key())
    }

    /// Returns an iterator over source identifiers for the given category.
    #[inline(always)]
    pub fn source_ids_for_category(&self, category: SourceCategory) -> impl Iterator<Item = SourceIdentifier> + '_ {
        self.sources.iter().filter(move |entry| entry.key().category() == category).map(|entry| *entry.key())
    }

    /// Returns an iterator over source identifiers for categories other than the given category.
    #[inline(always)]
    pub fn source_ids_except_category(&self, category: SourceCategory) -> impl Iterator<Item = SourceIdentifier> + '_ {
        self.sources.iter().filter(move |entry| entry.key().category() != category).map(|entry| *entry.key())
    }

    /// Loads the source with the given identifier.
    ///
    /// If the source content is already loaded, it is returned directly.
    /// Otherwise, the file is read from disk, processed, and cached.
    #[inline(always)]
    pub fn load(&self, source_id: &SourceIdentifier) -> Result<Source, SourceError> {
        // Try to get a mutable reference from the dashmap.
        let mut entry = self.sources.get_mut(source_id).ok_or(SourceError::UnavailableSource(*source_id))?;
        if let Some((content, size, ref lines)) = entry.content {
            // Fast path: content is already loaded.
            return Ok(Source {
                identifier: *source_id,
                path: entry.path.clone(),
                content,
                size,
                lines: lines.clone(),
            });
        }

        // Slow path: load from file.
        let path = entry.path.clone().expect("Entry must have either content or path");
        let bytes = std::fs::read(&path).map_err(SourceError::IOError)?;
        let content = match String::from_utf8_lossy(&bytes) {
            Cow::Borrowed(s) => s.to_owned(),
            Cow::Owned(s) => {
                tracing::warn!("Source '{}' contains invalid UTF-8 sequence, behavior is undefined.", path.display());

                s
            }
        };

        let lines: Vec<_> = line_starts(&content).collect();
        let size = content.len();
        let content_id = self.interner.intern(content);

        // Update the entry.
        entry.content = Some((content_id, size, lines.clone()));

        Ok(Source { identifier: *source_id, path: Some(path), content: content_id, size, lines })
    }

    /// Writes updated content for the source with the given identifier.
    #[inline(always)]
    pub fn write(&self, source_id: SourceIdentifier, new_content: impl AsRef<str>) -> Result<(), SourceError> {
        let mut entry = self.sources.get_mut(&source_id).ok_or(SourceError::UnavailableSource(source_id))?;
        let new_content = new_content.as_ref();
        let new_content_id = self.interner.intern(new_content);
        // Check if content is unchanged.
        if let Some((old_content, _, _)) = entry.content.as_ref() {
            if *old_content == new_content_id {
                return Ok(());
            }
        }

        let new_lines: Vec<_> = line_starts(new_content).collect();
        let new_size = new_content.len();

        entry.content = Some((new_content_id, new_size, new_lines.clone()));
        if let Some(ref path) = entry.path {
            std::fs::write(path, self.interner.lookup(&new_content_id)).map_err(SourceError::IOError)?;
        }

        Ok(())
    }

    /// Returns the number of sources.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Returns true if there are no sources.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

unsafe impl Send for SourceManager {}
unsafe impl Sync for SourceManager {}

impl<T: HasSource> HasSource for Box<T> {
    #[inline(always)]
    fn source(&self) -> SourceIdentifier {
        self.as_ref().source()
    }
}

/// Returns an iterator over the starting byte offsets of each line in `source`.
#[inline(always)]
fn line_starts(source: &str) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}
