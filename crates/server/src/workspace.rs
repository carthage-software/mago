//! Native workspace state shared by codebase services.
//!
//! Buffer keys belong to the caller. They may be paths, IDs, or protocol document keys;
//! the workspace never parses them. File lookup and edits use native paths and byte offsets.

use crate::Server;
use foldhash::HashMap;
use mago_database::Database;
use mago_database::file::{File, FileId};
use mago_database::membership::WorkspaceMatcher;
use mago_reporting::CompiledIgnoreSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod document;
pub use document::OpenDocument;
pub mod registry;

pub mod directory_aliases;
pub mod document_path;
pub mod paths;

use directory_aliases::DirectoryAliases;
use paths::builtin_file_at;

/// A loaded project, its path rules, and its accepted buffers.
pub struct Workspace<K = PathBuf> {
    pub folder: PathBuf,
    pub root: PathBuf,
    pub matcher: WorkspaceMatcher,
    pub directory_aliases: DirectoryAliases,
    pub server: Server,
    pub analyzer_ignores: CompiledIgnoreSet,
    source_root: Option<PathBuf>,
    opened: HashMap<FileId, OpenedFile<K>>,
}

struct OpenedFile<K> {
    key: K,
    path: PathBuf,
}

impl<K> Workspace<K> {
    /// Attach path rules to an already loaded server. This performs no analysis.
    #[must_use]
    pub fn new(
        folder: PathBuf,
        root: PathBuf,
        server: Server,
        matcher: WorkspaceMatcher,
        analyzer_ignores: CompiledIgnoreSet,
    ) -> Self {
        Self {
            folder,
            root,
            server,
            matcher,
            analyzer_ignores,
            directory_aliases: DirectoryAliases::default(),
            source_root: None,
            opened: HashMap::default(),
        }
    }

    pub fn database(&self) -> &Database<'static> {
        self.server.database()
    }
    pub fn database_mut(&mut self) -> &mut Database<'static> {
        self.server.database_mut()
    }

    /// Allow files published from the prelude to resolve back to builtin file IDs.
    pub fn set_source_root(&mut self, root: PathBuf) {
        self.source_root = Some(root);
    }

    pub fn builtin_file_at(&self, path: &Path) -> Option<Arc<File>> {
        builtin_file_at(self.database(), self.source_root.as_deref()?, path)
    }

    /// Record the exact buffer key and path whose contents this workspace accepted.
    fn open_file(&mut self, file: FileId, key: K, path: PathBuf) {
        self.opened.insert(file, OpenedFile { key, path });
    }

    fn close_file(&mut self, file: FileId) {
        self.opened.remove(&file);
    }
    pub fn opened_key(&self, file: FileId) -> Option<&K> {
        self.opened.get(&file).map(|file| &file.key)
    }

    /// Keep aliases queryable after their loaded path or symlink target disappears.
    pub fn opened_file_at(&self, path: &Path) -> Option<FileId> {
        self.opened.iter().find_map(|(file, opened)| (opened.path == path).then_some(*file))
    }
}

impl<K> AsRef<Workspace<K>> for Workspace<K> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<K> AsMut<Workspace<K>> for Workspace<K> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
