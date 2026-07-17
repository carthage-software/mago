//! `get_lens`: a "N references" lens above each top-level definition.
//!
//! The definition offsets come from the file's signature in the codebase; the
//! reference count reuses [`get_references`](crate::Server::get_references).

use mago_database::file::FileId;

use crate::Server;
use crate::domain::CodeLensItem;

impl Server {
    /// One [`CodeLensItem`] per top-level definition in `file_id`, each carrying
    /// the number of references to that definition.
    pub fn get_lens(&mut self, file_id: FileId) -> Vec<CodeLensItem> {
        let offsets: Vec<u32> = self
            .codebase()
            .file_signatures
            .get(&file_id)
            .map(|signature| signature.ast_nodes.iter().map(|node| node.start_offset).collect())
            .unwrap_or_default();

        offsets
            .into_iter()
            .map(|offset| CodeLensItem { offset, reference_count: self.get_references(file_id, offset, false).len() })
            .collect()
    }
}
