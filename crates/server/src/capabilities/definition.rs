//! `get_definition`: resolve the identifier under the cursor to its
//! fully-qualified name and look up that symbol's declaration span.

use mago_database::file::FileId;

use crate::Server;
use crate::domain::Range;
use crate::domain::SymbolLocation;

impl Server {
    /// The declaration location of the symbol whose identifier covers `offset`
    /// in `file_id`, or `None` if the offset isn't on a resolvable name.
    pub fn get_definition(&mut self, file_id: FileId, offset: u32) -> Option<SymbolLocation> {
        let analysis = self.file_analysis_for(file_id)?;
        let (_, _, fqcn, _) = analysis.resolved().at_offset(offset)?;
        let span = self.codebase().span_of(fqcn)?;
        Some(SymbolLocation { file: span.file_id, range: Range::new(span.start.offset, span.end.offset) })
    }
}
