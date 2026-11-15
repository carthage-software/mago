//! Import class names without changing other bindings.

use foldhash::HashMap;
use foldhash::HashSet;
use mago_allocator::LocalArena;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId, FileType};
use mago_linter::import_tracker::{ImportKind, ImportResolution, ImportTracker};
use mago_names::ResolvedNames;
use mago_names::resolver::NameResolver;
use mago_span::HasSpan;
use mago_syntax::cst::{Node, Program};
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::semantic_index::{self, Occurrence, SemanticIndex, SymbolKind};
use crate::{CodeActionItem, CodeActionKind, Range, Server, TextReplacement};

pub(super) fn import_classes(
    server: &Server,
    file_id: FileId,
    start: u32,
    end: u32,
    only: Option<&[&str]>,
) -> Option<Vec<CodeActionItem>> {
    let file = server.database().get(&file_id).ok()?;
    if file.file_type != FileType::Host || start > end || end > file.size {
        return None;
    }
    let source = std::str::from_utf8(&file.contents).ok()?;
    let arena = LocalArena::new();
    let program = parse_file_borrowed_with_settings(&arena, &file, server.parser_settings());
    if program.has_errors() {
        return None;
    }
    let names = NameResolver::new(&arena).resolve(program);
    let index = semantic_index::build(program, &names, &file.contents);
    let selected = index.occurrences.iter().find(|entry| {
        !entry.declaration
            && entry.range.start <= start
            && end <= entry.range.end
            && index.symbols[entry.symbol as usize].kind == SymbolKind::ClassLike
            && !index.imports.iter().any(|binding| binding.target_range == entry.range)
    })?;
    let raw = &source.as_bytes()[selected.range.start as usize..selected.range.end as usize];
    let mut allowed = HashSet::default();
    if let Some(fqn) = raw.strip_prefix(b"\\") {
        if !CodeActionKind::RefactorRewrite.requested(only) {
            return None;
        }
        return Some(action_for(server, &file, program, &names, &index, selected, fqn, &allowed).into_iter().collect());
    }
    if !CodeActionKind::QuickFix.requested(only)
        || selected.imported
        || raw.contains(&b'\\')
        || server.codebase().get_class_like(&index.symbols[selected.symbol as usize].name).is_some()
    {
        return None;
    }
    allowed.extend(
        index
            .occurrences
            .iter()
            .filter(|entry| {
                entry.symbol == selected.symbol
                    && !entry.declaration
                    && !entry.imported
                    && !source.as_bytes()[entry.range.start as usize..entry.range.end as usize].contains(&b'\\')
            })
            .map(|entry| entry.range.start),
    );
    let mut candidates: Vec<_> = server
        .codebase()
        .class_likes
        .values()
        .filter(|class| {
            class
                .original_name
                .as_bytes()
                .rsplit(|byte| *byte == b'\\')
                .next()
                .is_some_and(|name| name.eq_ignore_ascii_case(raw))
        })
        .collect();
    candidates.sort_unstable_by_key(|class| class.name.as_bytes());
    candidates.dedup_by_key(|class| class.name);
    Some(
        candidates
            .into_iter()
            .take(20)
            .filter_map(|class| {
                action_for(server, &file, program, &names, &index, selected, class.original_name.as_bytes(), &allowed)
            })
            .collect(),
    )
}

fn action_for(
    server: &Server,
    file: &File,
    program: &Program<'_>,
    names: &ResolvedNames<'_>,
    index: &SemanticIndex,
    selected: &Occurrence,
    fqn: &[u8],
    allowed: &HashSet<u32>,
) -> Option<CodeActionItem> {
    let source = std::str::from_utf8(&file.contents).ok()?;
    let quickfix = !allowed.is_empty();
    let (resolution, scope) = import_at(program, selected.range.start, fqn, quickfix)?;
    let local = std::str::from_utf8(resolution.local_name.as_bytes()).ok()?;
    let mut edits = Vec::new();
    if &source.as_bytes()[selected.range.start as usize..selected.range.end as usize] != local.as_bytes() {
        edits.push(TextReplacement { range: selected.range, new_text: local.to_owned() });
    }
    if let Some(edit) = resolution.use_statement_edit {
        // The name resolver does not cover PHPDoc types. Decline an import that could capture one.
        if program.trivia.iter().any(|trivia| {
            trivia.kind.is_docblock()
                && scope.contains(trivia.span.start.offset)
                && trivia
                    .value
                    .split(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'\\') || *byte >= 0x80))
                    .any(|name| {
                        name.split(|byte| *byte == b'\\')
                            .next()
                            .is_some_and(|first| first.eq_ignore_ascii_case(local.as_bytes()))
                    })
        }) {
            return None;
        }
        // A later import is not yet in the tracker, but still makes a duplicate invalid.
        if index.imports.iter().any(|binding| {
            binding.kind == SymbolKind::ClassLike
                && binding.namespace_range.contains(selected.range.start)
                && binding.alias.eq_ignore_ascii_case(local.as_bytes())
        }) {
            return None;
        }
        let mut offset = edit.range.start as usize;
        let text = std::str::from_utf8(&edit.new_text).ok()?;
        if text.starts_with('\n') {
            for comment in program.trivia.iter().filter(|trivia| trivia.kind.is_comment()) {
                let comment_start = comment.span.start.offset as usize;
                if comment_start < offset {
                    continue;
                }
                if !source.as_bytes()[offset..comment_start].iter().all(|byte| matches!(byte, b' ' | b'\t')) {
                    break;
                }
                offset = comment.span.end.offset as usize;
            }
        }
        let newline = if source.contains("\r\n") { "\r\n" } else { "\n" };
        let line_start = source.as_bytes()[..offset].iter().rposition(|byte| *byte == b'\n').map_or(0, |i| i + 1);
        let indent_len =
            source.as_bytes()[line_start..offset].iter().take_while(|byte| matches!(byte, b' ' | b'\t')).count();
        let indent = &source[line_start..line_start + indent_len];
        edits.push(TextReplacement {
            range: Range::new(offset as u32, offset as u32),
            new_text: text.replace('\n', &format!("{newline}{indent}")),
        });
    }
    edits.sort_unstable_by_key(|edit| (edit.range.start, edit.range.end));
    let mut changed = Vec::new();
    let mut cursor = 0;
    for edit in &edits {
        if edit.range.start < cursor || edit.range.end > file.size {
            return None;
        }
        changed.extend_from_slice(&file.contents[cursor as usize..edit.range.start as usize]);
        changed.extend_from_slice(edit.new_text.as_bytes());
        cursor = edit.range.end;
    }
    changed.extend_from_slice(&file.contents[cursor as usize..]);
    let changed_file = File::ephemeral(file.name.clone(), changed.into());
    let changed_arena = LocalArena::new();
    let changed_program = parse_file_borrowed_with_settings(&changed_arena, &changed_file, server.parser_settings());
    if changed_program.has_errors() {
        return None;
    }
    let changed_names = NameResolver::new(&changed_arena).resolve(changed_program);
    for (offset, _, name, _) in names.iter() {
        let shift: i64 = edits
            .iter()
            .filter(|edit| edit.range.end <= offset)
            .map(|edit| edit.new_text.len() as i64 - i64::from(edit.range.len()))
            .sum();
        let shifted = u32::try_from(i64::from(offset) + shift).ok()?;
        let changed = changed_names.resolve(&shifted)?;
        if offset == selected.range.start {
            if !changed.eq_ignore_ascii_case(fqn) {
                return None;
            }
        } else if !(changed.eq_ignore_ascii_case(name)
            || (allowed.contains(&offset) && changed.eq_ignore_ascii_case(fqn)))
        {
            return None;
        }
    }
    Some(CodeActionItem {
        title: if quickfix {
            format!("Import {}", String::from_utf8_lossy(fqn))
        } else {
            format!("Use {local} for \\{}", String::from_utf8_lossy(fqn))
        },
        kind: if quickfix { CodeActionKind::QuickFix } else { CodeActionKind::RefactorRewrite },
        edits: HashMap::from_iter([(file.id, edits)]),
        diagnostic: None,
    })
}

fn import_at(program: &Program<'_>, offset: u32, fqn: &[u8], quickfix: bool) -> Option<(ImportResolution, Range)> {
    let mut tracker = ImportTracker::new();
    let mut scope = Node::Program(program);
    let mut stack = vec![(scope, false)];
    while let Some((node, exit)) = stack.pop() {
        if exit {
            tracker.exit_node(node);
            continue;
        }
        tracker.enter_node(node);
        if matches!(node, Node::Namespace(_)) && node.start_offset() <= offset && offset < node.end_offset() {
            scope = node;
        }
        if matches!(node, Node::Identifier(_)) && node.start_offset() == offset {
            if !quickfix {
                tracker.collect_class_references(scope);
            }
            return tracker
                .import(fqn, ImportKind::Name)
                .map(|resolution| (resolution, Range::new(scope.start_offset(), scope.end_offset())));
        }
        stack.push((node, true));
        let first = stack.len();
        node.visit_children(|child| stack.push((child, false)));
        stack[first..].reverse();
    }
    None
}
