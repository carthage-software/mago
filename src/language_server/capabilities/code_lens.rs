//! `textDocument/codeLens`.
//!
//! Renders an unobtrusive "N references" line above every top-level
//! definition in the file. Reference counts come from the cross-file
//! references engine. The lens is unresolved; we don't ship a `command`
//! that runs anything; click handlers are the editor's responsibility,
//! and most editors invoke `textDocument/references` for these locations
//! anyway.

use mago_database::file::File as MagoFile;
use mago_database::file::FileId;
use tower_lsp::lsp_types::CodeLens;
use tower_lsp::lsp_types::Command;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Range;

use crate::language_server::state::WorkspaceState;

pub fn compute(workspace: &mut WorkspaceState, file: &MagoFile, file_id: FileId) -> Vec<CodeLens> {
    let nodes: Vec<(u32, u32, u32)> = workspace
        .service
        .codebase()
        .file_signatures
        .get(&file_id)
        .map(|sig| sig.ast_nodes.iter().map(|n| (n.start_offset, n.start_line, n.start_column as u32)).collect())
        .unwrap_or_default();

    nodes.into_iter().map(|(offset, line, column)| build_lens(workspace, file, offset, line, column)).collect()
}

fn build_lens(workspace: &mut WorkspaceState, file: &MagoFile, offset: u32, line: u32, column: u32) -> CodeLens {
    let count = crate::language_server::capabilities::references::compute(workspace, file, offset, false).len();
    let label = format!("{count} reference{}", if count == 1 { "" } else { "s" });

    CodeLens {
        range: Range {
            start: Position { line: line.saturating_sub(1), character: column },
            end: Position { line: line.saturating_sub(1), character: column },
        },
        command: Some(Command { title: label, command: String::new(), arguments: None }),
        data: None,
    }
}
