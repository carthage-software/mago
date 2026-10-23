//! Resolve call hierarchy items and collect edges from source files.

mod graph;

pub(crate) use graph::CallGraph;
use graph::GraphBuilder;

use foldhash::HashMap;
use mago_analyzer::artifacts::editor::EditorFileData;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_span::Position;
use mago_span::Span;
use mago_word::word;

use crate::Range;
use crate::Server;
use crate::SymbolKind;
use crate::call_analysis::{call_targets_with_callables, function_target, scope_at};
use crate::file_analysis::FileAnalysis;
use crate::semantic_index::ScopeKind;
use crate::symbol_resolution::methods_at;

/// A declaration and its key in the current workspace metadata.
#[derive(Debug, Clone)]
pub struct CallHierarchyItem {
    pub key: FunctionLikeIdentifier,
    pub name: String,
    pub kind: SymbolKind,
    pub file: FileId,
    pub range: Range,
    pub selection_range: Range,
}

/// One caller or callee, with all call positions in the caller's file.
#[derive(Debug, Clone)]
pub struct CallHierarchyCall {
    pub item: CallHierarchyItem,
    pub ranges: Vec<Range>,
}

impl Server {
    /// Find all host-file callers, building the reverse index once per workspace state.
    pub fn incoming_calls(&mut self, key: FunctionLikeIdentifier) -> Option<Vec<CallHierarchyCall>> {
        self.call_hierarchy_item(key)?;
        if self.call_graph.is_none() {
            let files: Vec<_> =
                self.database().files().filter(|file| file.file_type == FileType::Host).map(|file| file.id).collect();
            let mut builder = GraphBuilder::default();
            for file in files {
                self.collect_file_calls(file, &mut builder);
            }
            self.call_graph = Some(builder.finish());
        }
        Some(self.call_graph.as_ref()?.incoming(self.codebase(), key))
    }

    /// Find calls in one body; vendor bodies need only their own file parsed.
    pub fn outgoing_calls(&mut self, key: FunctionLikeIdentifier) -> Option<Vec<CallHierarchyCall>> {
        let item = self.call_hierarchy_item(key)?;
        let file = self.database().get(&item.file).ok()?;
        if file.file_type == FileType::Host
            && let Some(graph) = &self.call_graph
        {
            return Some(graph.outgoing(self.codebase(), key));
        }
        let mut builder = GraphBuilder::default();
        self.collect_file_calls(item.file, &mut builder);
        Some(builder.finish().outgoing(self.codebase(), key))
    }

    fn collect_file_calls(&mut self, file_id: FileId, builder: &mut GraphBuilder) {
        let Some(analysis) = self.file_analysis_for(file_id) else { return };
        let Ok(file) = self.database().get(&file_id) else { return };
        let editor = self.editor_file(file_id);
        collect_file_calls(self.codebase(), &file, &analysis, editor.as_deref(), builder);
    }

    /// Resolve a function or method declaration, call, or first-class callable.
    pub fn prepare_call_hierarchy(&mut self, file_id: FileId, offset: u32) -> Vec<CallHierarchyItem> {
        let Some(analysis) = self.file_analysis_for(file_id) else { return Vec::new() };
        let Ok(file) = self.database().get(&file_id) else { return Vec::new() };
        let editor = self.editor_file(file_id);
        let mut keys = Vec::new();
        let index = analysis.semantic();
        // A callable expression can contain other callable names. Prefer the name under the cursor.
        if let Some(occurrence) = index
            .at_offset(offset)
            .filter(|entry| index.symbols[entry.symbol as usize].kind == crate::semantic_index::SymbolKind::Function)
        {
            if occurrence.declaration {
                keys.push(FunctionLikeIdentifier::Function(word(&index.symbols[occurrence.symbol as usize].name)));
            } else {
                keys.extend(function_target(self.codebase(), &file, &analysis, occurrence.range));
            }
        } else if let Some(methods) =
            editor.as_ref().map(|editor| methods_at(editor, offset)).filter(|methods| !methods.is_empty())
        {
            keys.extend(methods.iter().filter(|row| !row.magic).map(|row| FunctionLikeIdentifier::from(row.method)));
        } else if let Some(call) =
            analysis.calls.iter().filter(|call| call.selection.contains(offset)).min_by_key(|call| call.selection.len())
        {
            keys.extend(call_targets_with_callables(self.codebase(), &file, &analysis, editor.as_deref(), call));
        }
        if keys.is_empty() {
            keys.extend(declarations(self.codebase(), file_id).filter(|key| {
                self.codebase()
                    .get_function_like(key)
                    .and_then(|meta| meta.name_span)
                    .is_some_and(|span| range(span).contains(offset))
            }));
            if keys.is_empty()
                && let Some(key) = owner_at(self.codebase(), &file, &analysis, offset).filter(|key| {
                    key.is_closure()
                        || self
                            .codebase()
                            .get_function_like(key)
                            .and_then(|meta| meta.name_span)
                            .is_some_and(|span| range(span).contains(offset))
                })
            {
                keys.push(key);
            }
        }
        keys.sort_unstable();
        keys.dedup();
        keys.into_iter().filter_map(|key| self.call_hierarchy_item(key)).collect()
    }

    /// Read the current declaration, so clients can reject stale hierarchy items.
    #[must_use]
    pub fn call_hierarchy_item(&self, key: FunctionLikeIdentifier) -> Option<CallHierarchyItem> {
        hierarchy_item(self.codebase(), key)
    }
}

fn hierarchy_item(codebase: &CodebaseMetadata, key: FunctionLikeIdentifier) -> Option<CallHierarchyItem> {
    let meta = codebase.get_function_like(&key)?;
    let name = if let FunctionLikeIdentifier::Method(class, _) = key {
        format!("{}::{}", codebase.get_class_like_by_name(class)?.original_name, meta.original_name)
    } else {
        meta.original_name.to_string()
    };
    Some(CallHierarchyItem {
        key,
        name,
        kind: if key.is_method() { SymbolKind::Method } else { SymbolKind::Function },
        file: meta.span.file_id,
        range: range(meta.span),
        selection_range: range(meta.name_span.unwrap_or(meta.span)),
    })
}

fn declarations(codebase: &CodebaseMetadata, file: FileId) -> impl Iterator<Item = FunctionLikeIdentifier> + '_ {
    codebase.file_signatures.get(&file).into_iter().flat_map(|signature| &signature.ast_nodes).flat_map(|node| {
        let function = node.is_function.then_some(FunctionLikeIdentifier::Function(node.name));
        function.into_iter().chain(
            node.children
                .iter()
                .filter(|child| child.is_function)
                .map(move |child| FunctionLikeIdentifier::Method(node.name, child.name)),
        )
    })
}

fn owner_at(
    codebase: &CodebaseMetadata,
    file: &File,
    analysis: &FileAnalysis,
    offset: u32,
) -> Option<FunctionLikeIdentifier> {
    let scope = scope_at(analysis, offset)?;
    match scope.kind {
        ScopeKind::Closure | ScopeKind::ArrowFunction => {
            let span = Span::new(file.id, Position::new(scope.range.start), Position::new(scope.range.end));
            Some(FunctionLikeIdentifier::for_closure(file, span))
        }
        ScopeKind::Function | ScopeKind::Method => declarations(codebase, file.id)
            .find(|key| {
                codebase
                    .get_function_like(key)
                    .is_some_and(|meta| meta.span.file_id == file.id && range(meta.span) == scope.range)
            })
            .or_else(|| {
                if scope.kind != ScopeKind::Method {
                    return None;
                }
                codebase.function_likes.iter().find_map(|((class, method), meta)| {
                    (meta.span.file_id == file.id && range(meta.span) == scope.range)
                        .then_some(FunctionLikeIdentifier::Method(*class, *method))
                })
            }),
        _ => None,
    }
}

fn collect_file_calls(
    codebase: &CodebaseMetadata,
    file: &File,
    analysis: &FileAnalysis,
    editor: Option<&EditorFileData>,
    builder: &mut GraphBuilder,
) {
    let mut owners: HashMap<_, _> = declarations(codebase, file.id)
        .filter_map(|key| {
            let meta = codebase.get_function_like(&key)?;
            (meta.span.file_id == file.id).then_some((meta.span.start.offset, key))
        })
        .collect();
    for scope in analysis.semantic().scopes.iter().filter(|scope| scope.kind == ScopeKind::Method) {
        if !owners.contains_key(&scope.range.start)
            && let Some(key) = owner_at(codebase, file, analysis, scope.range.start)
        {
            owners.insert(scope.range.start, key);
        }
    }
    for call in &analysis.calls {
        let Some(scope) = scope_at(analysis, call.expression.start) else { continue };
        let owner = if let Some(key) = owners.get(&scope.range.start) {
            *key
        } else if matches!(scope.kind, ScopeKind::Closure | ScopeKind::ArrowFunction) {
            let span = Span::new(file.id, Position::new(scope.range.start), Position::new(scope.range.end));
            let key = FunctionLikeIdentifier::for_closure(file, span);
            owners.insert(scope.range.start, key);
            key
        } else {
            continue;
        };
        for target in call_targets_with_callables(codebase, file, analysis, editor, call) {
            builder.add(owner, target, call.selection);
        }
    }
}

fn range(span: Span) -> Range {
    Range::new(span.start.offset, span.end.offset)
}

#[cfg(test)]
mod tests;
