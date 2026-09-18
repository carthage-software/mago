//! `get_selection_ranges`: nested AST-node span chains for each cursor.
//!
//! For each offset, collect the cached AST node spans that contain it, ordered
//! smallest to largest. The protocol layer turns each chain into a nested
//! selection range.

use mago_database::file::FileId;

use crate::Server;
use crate::domain::Range;
use crate::domain::SelectionRangeItem;

impl Server {
    /// For each offset, the chain of AST node spans covering it, innermost
    /// first. Returns one [`SelectionRangeItem`] per input offset.
    pub fn get_selection_ranges(&mut self, file_id: FileId, offsets: &[u32]) -> Vec<SelectionRangeItem> {
        let Some(analysis) = self.file_analysis_for(file_id) else {
            return offsets.iter().map(|_| SelectionRangeItem { ranges: Vec::new() }).collect();
        };

        offsets
            .iter()
            .map(|&offset| {
                let mut spans: Vec<(u32, u32)> =
                    analysis.node_spans.iter().copied().filter(|(s, e)| *s <= offset && offset <= *e).collect();
                spans.sort_by_key(|(s, e)| e - s);
                spans.dedup();
                SelectionRangeItem { ranges: spans.into_iter().map(|(s, e)| Range::new(s, e)).collect() }
            })
            .collect()
    }
}
