//! `textDocument/foldingRange`. Reads pre-computed fold ranges from the
//! file's [`super::super::file_analysis::FileAnalysis`].

use tower_lsp::lsp_types::FoldingRange;

use crate::language_server::file_analysis::FileAnalysis;

pub fn compute(analysis: &FileAnalysis) -> Vec<FoldingRange> {
    analysis.fold_ranges.clone()
}
