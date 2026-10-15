//! Preserve a loaded file's identity when a caller uses another path to it.

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use mago_database::DatabaseReader;
use mago_database::file::{File, FileType};

use super::super::Workspace;
use super::super::document_path::DocumentPath;
use super::super::paths::logical_name_for;

pub(super) enum FileTarget {
    Loaded(Arc<File>),
    New { path: PathBuf, file_type: FileType },
}

impl FileTarget {
    pub fn resolve<K>(workspace: &Workspace<K>, path: &DocumentPath<'_>) -> Option<Self> {
        Self::resolve_loaded(workspace, path, path.file(workspace))
    }

    pub fn resolve_loaded<K>(
        workspace: &Workspace<K>,
        path: &DocumentPath<'_>,
        file: Option<Arc<File>>,
    ) -> Option<Self> {
        if let Some(file) = file {
            return (!file.file_type.is_builtin()).then_some(Self::Loaded(file));
        }
        let file_type = workspace.matcher.classify(path.requested())?;
        let source_path = if !path.requested().components().any(|part| part == std::path::Component::ParentDir)
            && workspace.matcher.watch_roots().any(|root| path.requested().starts_with(root))
        {
            path.requested()
        } else {
            path.resolved()
        };
        Some(Self::New { path: source_path.to_path_buf(), file_type })
    }

    pub fn unchanged(&self, contents: &[u8]) -> bool {
        matches!(self, Self::Loaded(file) if file.contents == contents)
    }

    #[expect(clippy::expect_used, reason = "The exclusive database borrow keeps the file present after this write.")]
    pub fn write<K>(self, workspace: &mut Workspace<K>, contents: Cow<'static, [u8]>) -> Arc<File> {
        let id = match self {
            Self::Loaded(file) => {
                workspace.database_mut().update(file.id, contents);
                file.id
            }
            Self::New { path, file_type } => {
                let file = File::new(logical_name_for(&workspace.root, &path).into(), file_type, Some(path), contents);
                let id = file.id;
                workspace.database_mut().add(file);
                id
            }
        };
        workspace.database().get(&id).expect("file was written to the database")
    }
}
