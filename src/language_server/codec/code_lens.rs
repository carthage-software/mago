//! Reference lenses and commands. Unresolved lenses carry enough source data to reject stale positions.

use serde::{Deserialize, Serialize};
use tower_lsp_server::gen_lsp_types::{CodeLens, Command, Location, Position, Range, Uri};
use xxhash_rust::xxh3::xxh3_128;

use mago_database::file::File;
use mago_server::CodeLensItem;

use crate::language_server::position::position_at_offset;

#[derive(Deserialize, Serialize)]
pub struct CodeLensData {
    pub uri: Uri,
    content_hash: String,
}

impl CodeLensData {
    pub fn matches(&self, file: &File) -> bool {
        self.content_hash == format!("{:032x}", xxh3_128(&file.contents))
    }
}

pub fn code_lens(file: &File, uri: &Uri, items: impl IntoIterator<Item = CodeLensItem>) -> Vec<CodeLens> {
    let data = serde_json::json!(CodeLensData {
        uri: uri.clone(),
        content_hash: format!("{:032x}", xxh3_128(&file.contents)),
    });
    items
        .into_iter()
        .map(|item| {
            let position = position_at_offset(file, item.offset);
            CodeLens { range: Range { start: position, end: position }, command: None, data: Some(data.clone()) }
        })
        .collect()
}

pub fn reference_command(uri: &Uri, position: Position, locations: Vec<Location>) -> Command {
    let count = locations.len();
    Command {
        tooltip: None,
        title: format!("{count} reference{}", if count == 1 { "" } else { "s" }),
        // Zed and VS Code handle this client command; LSP does not define a references-view command.
        command: "editor.action.showReferences".into(),
        arguments: Some(vec![serde_json::json!(uri), serde_json::json!(position), serde_json::json!(locations)]),
    }
}
