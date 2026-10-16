use serde::Deserialize;
use serde::Serialize;
use tower_lsp_server::gen_lsp_types::CallHierarchyItem;
use tower_lsp_server::gen_lsp_types::Range;
use tower_lsp_server::gen_lsp_types::SymbolKind;
use tower_lsp_server::gen_lsp_types::Uri;

use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_database::DatabaseReader;
use mago_server::capabilities::call_hierarchy::CallHierarchyItem as Item;

use crate::language_server::position::range_at_offsets;
use crate::language_server::state::WorkspaceState;
use crate::language_server::uri::UriFilePath;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    pub workspace: Uri,
    key: FunctionLikeIdentifier,
}

pub struct RelatedCall {
    pub item: CallHierarchyItem,
    pub ranges: Vec<Range>,
}

pub fn data(item: &CallHierarchyItem) -> Option<Data> {
    serde_json::from_value(item.data.clone()?).ok()
}

pub fn item(workspace: &mut WorkspaceState, item: Item) -> Option<CallHierarchyItem> {
    let file = workspace.database().get(&item.file).ok()?;
    let data = Data { workspace: Uri::from_file_path(&workspace.core.root)?, key: item.key };
    Some(CallHierarchyItem {
        name: item.name,
        kind: if item.kind == mago_server::SymbolKind::Method { SymbolKind::Method } else { SymbolKind::Function },
        tags: None,
        detail: None,
        uri: workspace.uri(&file)?,
        range: range_at_offsets(&file, item.range.start, item.range.end),
        selection_range: range_at_offsets(&file, item.selection_range.start, item.selection_range.end),
        data: Some(serde_json::to_value(data).ok()?),
    })
}

#[derive(Clone, Copy)]
pub enum Direction {
    Incoming,
    Outgoing,
}

pub fn related(
    workspace: &mut WorkspaceState,
    original: &CallHierarchyItem,
    data: &Data,
    direction: Direction,
) -> Option<Vec<RelatedCall>> {
    if Uri::from_file_path(&workspace.core.root).as_ref() != Some(&data.workspace) {
        return None;
    }
    let current = workspace.core.server.call_hierarchy_item(data.key)?;
    let source = current.file;
    if item(workspace, current).as_ref() != Some(original) {
        return None;
    }
    let incoming = matches!(direction, Direction::Incoming);
    let calls = if incoming {
        workspace.core.server.incoming_calls(data.key)
    } else {
        workspace.core.server.outgoing_calls(data.key)
    }?;
    Some(
        calls
            .into_iter()
            .filter_map(|call| {
                let file = workspace.database().get(&if incoming { call.item.file } else { source }).ok()?;
                let ranges =
                    call.ranges.into_iter().map(|range| range_at_offsets(&file, range.start, range.end)).collect();
                Some(RelatedCall { item: item(workspace, call.item)?, ranges })
            })
            .collect(),
    )
}
