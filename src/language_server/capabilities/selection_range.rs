//! `textDocument/selectionRange`. Builds the parent-chain LSP response
//! by intersecting cursor offsets with cached AST node spans
//! ([`super::super::file_analysis::FileAnalysis::node_spans`]).

use mago_database::file::File as MagoFile;
use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::SelectionRange;

use crate::language_server::file_analysis::FileAnalysis;
use crate::language_server::position::range_at_offsets;

pub fn compute(analysis: &FileAnalysis, file: &MagoFile, offsets: &[u32]) -> Vec<SelectionRange> {
    offsets.iter().map(|&offset| compute_one(analysis, file, offset)).collect()
}

fn compute_one(analysis: &FileAnalysis, file: &MagoFile, offset: u32) -> SelectionRange {
    let mut spans: Vec<(u32, u32)> =
        analysis.node_spans.iter().copied().filter(|(s, e)| *s <= offset && offset <= *e).collect();
    spans.sort_by_key(|(s, e)| e - s);
    spans.dedup();

    let mut current: Option<SelectionRange> = None;
    for (start, end) in spans.into_iter().rev() {
        current = Some(SelectionRange { range: range_at_offsets(file, start, end), parent: current.map(Box::new) });
    }
    current.unwrap_or(SelectionRange { range: Range::default(), parent: None })
}
