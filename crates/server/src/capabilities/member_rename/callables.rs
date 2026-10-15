//! Check runtime callable names that ordinary member references cannot rename.

use foldhash::HashSet;
use mago_allocator::LocalArena;
use mago_analyzer::artifacts::editor::EditorFileData;
use mago_codex::metadata::CodebaseMetadata;
use mago_database::file::File;
use mago_syntax_core::utils::parse_literal_string_in;
use mago_word::Word;

use crate::semantic_index::{CallableReceiver, CallableReference, SemanticIndex, SymbolKind};
use crate::{Range, RefusalReason, RenameRefusal};

use super::{Kind, Selection, same_name};

pub(super) fn check(
    codebase: &CodebaseMetadata,
    file: &File,
    index: &SemanticIndex,
    editor: Option<&EditorFileData>,
    selected: &Selection,
    replacement: Option<Word>,
    affected: &HashSet<Word>,
) -> Result<(), RenameRefusal> {
    if selected.kind != Kind::Method || index.callable_references.is_empty() {
        return Ok(());
    }
    let arena = LocalArena::new();
    for callable in &index.callable_references {
        let refusal = |reason| RenameRefusal::at(reason, file.id, callable.method);
        let literal = literal_value(&arena, file, callable.method).ok_or(refusal(RefusalReason::Stale))?;
        let name = literal.rsplit(|byte| *byte == b':').next().unwrap_or(literal);
        let old = same_name(Kind::Method, name, selected.name.as_bytes());
        let new = replacement.is_some_and(|replacement| same_name(Kind::Method, name, replacement.as_bytes()));
        if !old && !new {
            continue;
        }
        let classes = callable_classes(codebase, file, index, editor, callable, &arena)
            .ok_or(refusal(RefusalReason::Unresolved))?;
        if classes.iter().any(|class| {
            affected.iter().any(|affected| {
                codebase.is_instance_of(affected.as_bytes(), class.as_bytes())
                    || codebase.class_uses_trait(affected.as_bytes(), class.as_bytes())
            })
        }) {
            return Err(refusal(RefusalReason::StringReference { proposed: !old }));
        }
    }
    Ok(())
}

fn literal_value<'a>(arena: &'a LocalArena, file: &'a File, range: Range) -> Option<&'a [u8]> {
    parse_literal_string_in(arena, file.contents.get(range.start as usize..range.end as usize)?, None, true)
}

fn callable_classes(
    codebase: &CodebaseMetadata,
    file: &File,
    index: &SemanticIndex,
    editor: Option<&EditorFileData>,
    callable: &CallableReference,
    arena: &LocalArena,
) -> Option<Vec<Word>> {
    if matches!(callable.receiver, Some(CallableReceiver::Unknown)) {
        return None;
    }
    let literal = literal_value(arena, file, callable.method)?;
    let explicit = literal.windows(2).rposition(|bytes| bytes == b"::").map(|position| &literal[..position]);
    let raw = |range: Range| file.contents.get(range.start as usize..range.end as usize);
    let named = |name: &[u8]| {
        let name = name.strip_prefix(b"\\").unwrap_or(name);
        if name.eq_ignore_ascii_case(b"self")
            || name.eq_ignore_ascii_case(b"static")
            || name.eq_ignore_ascii_case(b"parent")
        {
            let owner = callable.owner?;
            let class = codebase
                .class_likes
                .values()
                .find(|class| class.span.file_id == file.id && class.span.start.offset == owner)?;
            return Some(if name.eq_ignore_ascii_case(b"parent") {
                class.direct_parent_class.into_iter().collect()
            } else {
                vec![class.name]
            });
        }
        Some(codebase.get_class_like(name).map(|class| class.name).into_iter().collect())
    };
    if let Some(name) = explicit {
        return named(name);
    }
    match callable.receiver? {
        CallableReceiver::Literal(range) => named(literal_value(arena, file, range)?),
        CallableReceiver::ClassName(range) => {
            let name = raw(range)?;
            if name.eq_ignore_ascii_case(b"self")
                || name.eq_ignore_ascii_case(b"static")
                || name.eq_ignore_ascii_case(b"parent")
            {
                return named(name);
            }
            if let Some(occurrence) = index.at_offset(range.start)
                && occurrence.range == range
                && index.symbols[occurrence.symbol as usize].kind == SymbolKind::ClassLike
            {
                return named(&index.symbols[occurrence.symbol as usize].name);
            }
            expression_classes(editor?, range)
        }
        CallableReceiver::Expression(range) => expression_classes(editor?, range),
        CallableReceiver::Unknown => None,
    }
}

fn expression_classes(editor: &EditorFileData, range: Range) -> Option<Vec<Word>> {
    let index = editor.expression_classes.binary_search_by_key(&(range.start, range.end), |row| row.span).ok()?;
    let row = &editor.expression_classes[index];
    (!row.uncertain && !row.classes.is_empty())
        .then(|| row.classes.iter().map(|class| class.to_ascii_lowercase()).collect())
}
