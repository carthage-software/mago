//! [`File`] adapter over [`mago_database::file::File`].
//!
//! Grit's [`File`] trait is lifetime-parameterised on `'a`, but a [`mago_database::file::File`]
//! owns its name and contents (`Cow<'static, _>`). This module exposes [`MagoFile`], a thin
//! borrowed handle over a database file that satisfies the trait without copying.

use grit_pattern_matcher::pattern::File;
use grit_pattern_matcher::pattern::FileRegistry;
use grit_pattern_matcher::pattern::ResolvedPattern;
use grit_util::error::GritResult;

use crate::query_context::MagoQueryContext;
use crate::resolved_pattern::MagoResolvedPattern;

/// Borrowed view over a [`mago_database::file::File`] for the grit engine.
#[derive(Debug, Clone, Copy)]
pub struct MagoFile<'a> {
    file: &'a mago_database::file::File,
}

impl<'a> MagoFile<'a> {
    pub fn new(file: &'a mago_database::file::File) -> Self {
        Self { file }
    }

    pub fn inner(&self) -> &'a mago_database::file::File {
        self.file
    }
}

impl<'a> PartialEq for MagoFile<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.file.id == other.file.id
    }
}

impl<'a> File<'a, MagoQueryContext> for MagoFile<'a> {
    fn name(&self, _files: &FileRegistry<'a, MagoQueryContext>) -> MagoResolvedPattern<'a> {
        MagoResolvedPattern::from_string(self.file.name.as_ref().to_string())
    }

    fn absolute_path(
        &self,
        _files: &FileRegistry<'a, MagoQueryContext>,
        _language: &<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Language<'a>,
    ) -> GritResult<MagoResolvedPattern<'a>> {
        let absolute = self
            .file
            .path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.file.name.as_ref().to_string());
        Ok(MagoResolvedPattern::from_string(absolute))
    }

    fn body(&self, _files: &FileRegistry<'a, MagoQueryContext>) -> MagoResolvedPattern<'a> {
        MagoResolvedPattern::from_string(self.file.contents.as_ref().to_string())
    }

    fn binding(&self, _files: &FileRegistry<'a, MagoQueryContext>) -> MagoResolvedPattern<'a> {
        MagoResolvedPattern::from_string(self.file.contents.as_ref().to_string())
    }
}
