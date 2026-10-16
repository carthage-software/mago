//! Reference lenses and commands. Unresolved lenses name their declaration so they survive edits.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp_server::gen_lsp_types::{CodeLens, Command, Location, Position, Range, Uri};

use mago_database::file::File;
use mago_server::{CodeLensItem, CodeLensKind};

use crate::language_server::position::position_at_offset;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Data<'a> {
    uri: Cow<'a, Uri>,
    #[serde(with = "Kind")]
    kind: CodeLensKind,
    name: Vec<u8>,
    offset: u32,
    from_end: u32,
    ordinal: u32,
    siblings: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(remote = "CodeLensKind", rename_all = "snake_case")]
enum Kind {
    ClassLike,
    Function,
    Constant,
}

pub fn code_lens(file: &File, uri: &Uri, items: impl IntoIterator<Item = CodeLensItem>) -> Vec<CodeLens> {
    items.into_iter().map(|item| unresolved_lens(file, uri, item)).collect()
}

pub fn unresolved_lens(file: &File, uri: &Uri, item: CodeLensItem) -> CodeLens {
    let position = position_at_offset(file, item.offset);
    let data = Data {
        uri: Cow::Borrowed(uri),
        kind: item.kind,
        name: item.name,
        offset: item.offset,
        from_end: item.from_end,
        ordinal: item.ordinal,
        siblings: item.siblings,
    };
    CodeLens { range: Range { start: position, end: position }, command: None, data: Some(serde_json::json!(data)) }
}

pub fn lens_target(data: Value) -> Option<(Uri, CodeLensItem)> {
    let data: Data<'_> = serde_json::from_value(data).ok()?;
    let item = CodeLensItem {
        offset: data.offset,
        from_end: data.from_end,
        kind: data.kind,
        name: data.name,
        ordinal: data.ordinal,
        siblings: data.siblings,
    };
    Some((data.uri.into_owned(), item))
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
