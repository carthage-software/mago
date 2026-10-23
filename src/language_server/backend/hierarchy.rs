//! Prepare hierarchy items and route their follow-up requests to the owning workspace.

use tower_lsp_server::gen_lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall, Position, TypeHierarchyItem,
};
use tower_lsp_server::jsonrpc::Result;

use crate::config::language_server::Provider;
use crate::language_server::capabilities::{call_hierarchy, type_hierarchy};

use super::Backend;
use super::query::DocumentQuery;

impl DocumentQuery<'_> {
    pub fn call_hierarchy(self, position: Position) -> Option<Vec<CallHierarchyItem>> {
        let offset = self.checked_symbol_offset(position)?;
        let items = self.workspace.core.server.prepare_call_hierarchy(self.file.id, offset);
        let items: Vec<_> = items.into_iter().filter_map(|value| call_hierarchy::item(self.workspace, value)).collect();
        (!items.is_empty()).then_some(items)
    }

    pub fn type_hierarchy(self, position: Position) -> Option<Vec<TypeHierarchyItem>> {
        let offset = self.checked_symbol_offset(position)?;
        let item = self.workspace.core.server.prepare_type_hierarchy(self.file.id, offset)?;
        type_hierarchy::item(self.workspace, item).map(|item| vec![item])
    }
}

impl Backend {
    pub(super) fn incoming_calls_for(&self, item: CallHierarchyItem) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        self.related_calls(item, call_hierarchy::Direction::Incoming).map(|calls| {
            calls.map(|calls| {
                calls
                    .into_iter()
                    .map(|call| CallHierarchyIncomingCall { from: call.item, from_ranges: call.ranges })
                    .collect()
            })
        })
    }

    pub(super) fn outgoing_calls_for(&self, item: CallHierarchyItem) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        self.related_calls(item, call_hierarchy::Direction::Outgoing).map(|calls| {
            calls.map(|calls| {
                calls
                    .into_iter()
                    .map(|call| CallHierarchyOutgoingCall { to: call.item, from_ranges: call.ranges })
                    .collect()
            })
        })
    }

    fn related_calls(
        &self,
        item: CallHierarchyItem,
        direction: call_hierarchy::Direction,
    ) -> Result<Option<Vec<call_hierarchy::RelatedCall>>> {
        let Some(data) = call_hierarchy::data(&item) else { return Ok(None) };
        self.workspace_query(Provider::CallHierarchy, &data.workspace, |workspace| {
            call_hierarchy::related(workspace, &item, &data, direction)
        })
    }

    pub(super) fn related_types(
        &self,
        item: TypeHierarchyItem,
        direction: type_hierarchy::Direction,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let Some(data) = type_hierarchy::data(&item) else { return Ok(None) };
        self.workspace_query(Provider::TypeHierarchy, &data.workspace, |workspace| {
            type_hierarchy::related(workspace, &item, &data, direction)
        })
    }
}
