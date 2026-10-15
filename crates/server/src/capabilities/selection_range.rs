//! Expand each cursor from its token through expressions, statements, and declarations.

use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_syntax::token::TokenKind;

use crate::Server;
use crate::domain::Range;
use crate::domain::SelectionRangeItem;

impl Server {
    /// Return one strictly nested range chain per cursor, innermost first.
    pub fn get_selection_ranges(&mut self, file_id: FileId, offsets: &[u32]) -> Vec<SelectionRangeItem> {
        let Some(analysis) = self.file_analysis_for(file_id) else {
            return offsets.iter().map(|_| SelectionRangeItem { ranges: Vec::new() }).collect();
        };
        let Ok(file) = self.database().get(&file_id) else {
            return offsets.iter().map(|_| SelectionRangeItem { ranges: Vec::new() }).collect();
        };
        let tokens = self.lex(&file);

        offsets
            .iter()
            .map(|&offset| {
                let offset = offset.min(file.size);
                let covers = |start, end| start <= offset && (offset < end || offset == file.size && offset == end);
                let token = tokens
                    .partition_point(|token| token.start.offset <= offset)
                    .checked_sub(1)
                    .and_then(|index| tokens.get(index))
                    .filter(|token| {
                        token.kind != TokenKind::Whitespace
                            && !token.value.is_empty()
                            && covers(token.start.offset, token.start.offset + token.value.len() as u32)
                    });
                let leaf = token.map_or(Range::new(offset, offset), |token| {
                    Range::new(token.start.offset, token.start.offset + token.value.len() as u32)
                });
                let mut spans: Vec<_> = analysis
                    .node_spans
                    .iter()
                    .chain(analysis.expression_spans.iter())
                    .copied()
                    .chain(std::iter::once((0, file.size)))
                    .filter(|(start, end)| covers(*start, *end))
                    .collect();
                spans.sort_unstable_by_key(|(start, end)| (end - start, *start, *end));
                spans.dedup();
                let mut ranges = vec![leaf];
                let mut child = leaf;
                for (start, end) in spans {
                    if start <= child.start && child.end <= end && (start != child.start || end != child.end) {
                        child = Range::new(start, end);
                        ranges.push(child);
                    }
                }
                SelectionRangeItem { ranges }
            })
            .collect()
    }
}
