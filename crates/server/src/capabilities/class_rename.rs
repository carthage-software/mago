//! Check literal callable class names before renaming a class.

use mago_allocator::LocalArena;
use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_syntax_core::utils::parse_literal_string_in;

use crate::capabilities::rename::STALE;
use crate::{RefusalReason, RenameRefusal, Server, SymbolLocation};

impl Server {
    /// Find a runtime callable string naming one of `names`, and which name it matched.
    ///
    /// # Errors
    ///
    /// Returns a [`RenameRefusal`] when a callable string cannot be decoded.
    pub fn callable_class_reference(
        &mut self,
        names: &[&[u8]],
    ) -> Result<Option<(SymbolLocation, usize)>, RenameRefusal> {
        let files: Vec<_> = self.database().files().filter(|file| file.file_type == FileType::Host).collect();
        for file in files {
            let analysis = self.file_analysis_for(file.id).ok_or(STALE)?;
            let index = analysis.semantic();
            if index.callable_class_literals.is_empty() && index.callable_references.is_empty() {
                continue;
            }
            let arena = LocalArena::new();
            let matched = |name: &[u8]| {
                let name = name.strip_prefix(b"\\").unwrap_or(name);
                names.iter().position(|candidate| name.eq_ignore_ascii_case(candidate))
            };
            let literal = |range: crate::Range| {
                file.contents
                    .get(range.start as usize..range.end as usize)
                    .and_then(|raw| parse_literal_string_in(&arena, raw, None, true))
                    .ok_or(RenameRefusal::at(RefusalReason::Stale, file.id, range))
            };
            for range in &index.callable_class_literals {
                if let Some(name) = matched(literal(*range)?) {
                    return Ok(Some((SymbolLocation { file: file.id, range: *range }, name)));
                }
            }
            for callable in &index.callable_references {
                let literal = literal(callable.method)?;
                if let Some(separator) = literal.windows(2).position(|bytes| bytes == b"::")
                    && let Some(name) = matched(&literal[..separator])
                {
                    return Ok(Some((SymbolLocation { file: file.id, range: callable.method }, name)));
                }
            }
        }
        Ok(None)
    }
}
