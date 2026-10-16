//! Store call edges once and index both query directions.

use foldhash::HashMap;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;

use super::{CallHierarchyCall, hierarchy_item};
use crate::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CallEdge {
    from: u32,
    to: u32,
    range: Range,
}

/// Each edge takes 16 bytes; its reverse index takes another four.
pub(crate) struct CallGraph {
    keys: Box<[FunctionLikeIdentifier]>,
    edges: Box<[CallEdge]>,
    incoming: Box<[u32]>,
}

#[derive(Default)]
pub(super) struct GraphBuilder {
    nodes: HashMap<FunctionLikeIdentifier, u32>,
    edges: Vec<CallEdge>,
}

impl GraphBuilder {
    pub(super) fn add(&mut self, from: FunctionLikeIdentifier, to: FunctionLikeIdentifier, range: Range) {
        let next = self.nodes.len() as u32;
        let from = *self.nodes.entry(from).or_insert(next);
        let next = self.nodes.len() as u32;
        let to = *self.nodes.entry(to).or_insert(next);
        self.edges.push(CallEdge { from, to, range });
    }

    pub(super) fn finish(mut self) -> CallGraph {
        let mut nodes: Vec<_> = self.nodes.into_iter().collect();
        nodes.sort_unstable_by_key(|(key, _)| *key);
        let mut remap = vec![0; nodes.len()];
        for (index, (_, old)) in nodes.iter().enumerate() {
            remap[*old as usize] = index as u32;
        }
        for edge in &mut self.edges {
            edge.from = remap[edge.from as usize];
            edge.to = remap[edge.to as usize];
        }
        self.edges.sort_unstable();
        self.edges.dedup();
        let mut incoming: Vec<_> = (0..self.edges.len() as u32).collect();
        incoming.sort_unstable_by_key(|index| {
            let edge = self.edges[*index as usize];
            (edge.to, edge.from, edge.range)
        });
        CallGraph {
            keys: nodes.into_iter().map(|(key, _)| key).collect(),
            edges: self.edges.into_boxed_slice(),
            incoming: incoming.into_boxed_slice(),
        }
    }
}

impl CallGraph {
    pub(super) fn incoming(&self, codebase: &CodebaseMetadata, key: FunctionLikeIdentifier) -> Vec<CallHierarchyCall> {
        self.related(codebase, key, true)
    }

    pub(super) fn outgoing(&self, codebase: &CodebaseMetadata, key: FunctionLikeIdentifier) -> Vec<CallHierarchyCall> {
        self.related(codebase, key, false)
    }

    fn related(
        &self,
        codebase: &CodebaseMetadata,
        key: FunctionLikeIdentifier,
        incoming: bool,
    ) -> Vec<CallHierarchyCall> {
        let Ok(id) = self.keys.binary_search(&key) else { return Vec::new() };
        let id = id as u32;
        let edge = |index: usize| {
            if incoming { self.edges[self.incoming[index] as usize] } else { self.edges[index] }
        };
        let target = |index| if incoming { edge(index).to } else { edge(index).from };
        let indices = if incoming { self.incoming.as_ref() } else { &[] };
        let start = if incoming {
            indices.partition_point(|index| self.edges[*index as usize].to < id)
        } else {
            self.edges.partition_point(|edge| edge.from < id)
        };
        let mut groups: Vec<(u32, Vec<Range>)> = Vec::new();
        for index in (start..self.edges.len()).take_while(|index| target(*index) == id) {
            let edge = edge(index);
            let related = if incoming { edge.from } else { edge.to };
            if let Some((last, ranges)) = groups.last_mut()
                && *last == related
            {
                ranges.push(edge.range);
            } else {
                groups.push((related, vec![edge.range]));
            }
        }
        groups
            .into_iter()
            .filter_map(|(id, ranges)| {
                Some(CallHierarchyCall { item: hierarchy_item(codebase, self.keys[id as usize])?, ranges })
            })
            .collect()
    }
}
