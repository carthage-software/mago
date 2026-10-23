use serde::Deserialize;
use serde::Serialize;
use tower_lsp_server::gen_lsp_types::SymbolKind;
use tower_lsp_server::gen_lsp_types::TypeHierarchyItem;
use tower_lsp_server::gen_lsp_types::Uri;

use mago_database::DatabaseReader;
use mago_server::capabilities::type_hierarchy::TypeHierarchyItem as Item;

use crate::language_server::position::range_at_offsets;
use crate::language_server::state::WorkspaceState;
use crate::language_server::uri::UriFilePath;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    pub workspace: Uri,
    key: Vec<u8>,
}

pub fn data(item: &TypeHierarchyItem) -> Option<Data> {
    let data: Data = serde_json::from_value(item.data.clone()?).ok()?;
    (!data.key.is_empty()).then_some(data)
}

pub fn item(workspace: &mut WorkspaceState, item: Item) -> Option<TypeHierarchyItem> {
    let file = workspace.database().get(&item.file).ok()?;
    let data = Data { workspace: Uri::from_file_path(&workspace.core.root)?, key: item.key };
    Some(TypeHierarchyItem {
        name: item.name,
        kind: match item.kind {
            mago_server::SymbolKind::Interface => SymbolKind::Interface,
            mago_server::SymbolKind::Enum => SymbolKind::Enum,
            _ => SymbolKind::Class,
        },
        tags: None,
        detail: None,
        uri: workspace.uri(&file)?,
        range: range_at_offsets(&file, item.range.start, item.range.end),
        selection_range: range_at_offsets(&file, item.selection_range.start, item.selection_range.end),
        data: Some(serde_json::to_value(data).ok()?),
    })
}

pub enum Direction {
    Supertypes,
    Subtypes,
}

pub fn related(
    workspace: &mut WorkspaceState,
    original: &TypeHierarchyItem,
    data: &Data,
    direction: Direction,
) -> Option<Vec<TypeHierarchyItem>> {
    if Uri::from_file_path(&workspace.core.root).as_ref() != Some(&data.workspace) {
        return None;
    }
    let current = workspace.core.server.type_hierarchy_item(&data.key)?;
    if item(workspace, current).as_ref() != Some(original) {
        return None;
    }
    let items = if matches!(direction, Direction::Supertypes) {
        workspace.core.server.type_supertypes(&data.key)
    } else {
        workspace.core.server.type_subtypes(&data.key)
    }?;
    Some(items.into_iter().filter_map(|value| item(workspace, value)).collect())
}
