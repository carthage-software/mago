//! Collect checked member uses and reject ambiguous edits or name capture.

use foldhash::HashSet;
use mago_analyzer::artifacts::editor::MemberKind;
use mago_database::file::File;
use mago_span::Span;
use mago_word::Word;

use crate::capabilities::rename::STALE;
use crate::semantic_index::SelectorKind;
use crate::symbol_resolution::{members_at, methods_at};
use crate::{RefusalReason, RenameRefusal, Server, SymbolLocation};

use super::family::Family;
use super::{Kind, Selection, callables, checked_range, same_name, use_problem};

pub(super) fn collect(
    server: &mut Server,
    file: &File,
    selected: &Selection,
    replacement: Option<Word>,
    family: &Family,
    locations: &mut Vec<SymbolLocation>,
) -> Result<(), RenameRefusal> {
    let affected = &family.classes;
    let declarations = &family.declarations;
    let analysis = server.file_analysis_for(file.id).ok_or(STALE)?;
    let editor = server.editor_file(file.id);
    callables::check(server.codebase(), file, analysis.semantic(), editor.as_deref(), selected, replacement, affected)?;
    for selector in &analysis.semantic().member_selectors {
        let refusal = |reason| RenameRefusal::at(reason, file.id, selector.range);
        let text = &file.contents[selector.range.start as usize..selector.range.end as usize];
        if matches!(selector.kind, SelectorKind::TraitAdaptation | SelectorKind::TraitAlias) {
            if selected.kind == Kind::Method
                && same_name(selected.kind, text, selected.name.as_bytes())
                && selector.owner.is_some_and(|owner| {
                    server.codebase().class_likes.values().any(|class| {
                        class.span.file_id == file.id
                            && class.span.start.offset == owner
                            && affected.contains(&class.name)
                    })
                })
            {
                return Err(refusal(RefusalReason::TraitAdaptation));
            }
            continue;
        }
        let kind = match selector.kind {
            SelectorKind::Method => Kind::Method,
            SelectorKind::Property => Kind::Property,
            SelectorKind::Constant => Kind::Constant,
            SelectorKind::TraitAdaptation | SelectorKind::TraitAlias => continue,
        };
        if kind == selected.kind
            && replacement.is_some_and(|name| same_name(kind, text, name.as_bytes()))
            && !same_name(kind, text, selected.name.as_bytes())
        {
            let editor = editor.as_ref().ok_or(refusal(RefusalReason::AnalysisDisabled))?;
            let captures = if kind == Kind::Method {
                let rows = methods_at(editor, selector.range.start);
                rows.is_empty()
                    || rows
                        .iter()
                        .any(|row| row.uncertain || affected.contains(&row.receiver_class.to_ascii_lowercase()))
            } else {
                let rows = members_at(editor, selector.range.start);
                rows.is_empty()
                    || rows.iter().any(|row| {
                        row.uncertain
                            || row.lookup_class.is_none_or(|class| affected.contains(&class.to_ascii_lowercase()))
                    })
            };
            if captures {
                return Err(refusal(RefusalReason::Capture));
            }
            continue;
        }
        if kind != selected.kind || (!selector.dynamic && !same_name(kind, text, selected.name.as_bytes())) {
            continue;
        }
        let editor = editor.as_ref().ok_or(refusal(RefusalReason::AnalysisDisabled))?;
        let state = if kind == Kind::Method {
            reference_state(
                methods_at(editor, selector.range.start)
                    .iter()
                    .map(|row| (row.declaration, use_problem(row.dynamic, row.magic, row.uncertain))),
                declarations,
            )
        } else {
            reference_state(
                members_at(editor, selector.range.start)
                    .iter()
                    .filter(|row| (row.kind == MemberKind::Property) == (kind == Kind::Property))
                    .map(|row| (row.declaration, use_problem(row.dynamic, row.magic, row.uncertain))),
                declarations,
            )
        };
        if !state.complete {
            return Err(refusal(state.problem.unwrap_or(RefusalReason::Unresolved)));
        }
        if !state.touches {
            continue;
        }
        if selector.dynamic {
            return Err(refusal(RefusalReason::Dynamic));
        }
        if !state.safe {
            return Err(refusal(RefusalReason::SharedTarget));
        }
        locations.push(SymbolLocation {
            file: file.id,
            range: checked_range(file, selector.range, selected.name.as_bytes(), selected.kind)
                .ok_or(refusal(RefusalReason::Stale))?,
        });
    }
    Ok(())
}

struct ReferenceState {
    touches: bool,
    safe: bool,
    complete: bool,
    problem: Option<RefusalReason>,
}

fn reference_state(
    rows: impl Iterator<Item = (Option<Span>, Option<RefusalReason>)>,
    family: &HashSet<Span>,
) -> ReferenceState {
    let mut state = ReferenceState { touches: false, safe: true, complete: true, problem: None };
    let mut seen = false;
    for (span, problem) in rows {
        seen = true;
        let member = span.is_some_and(|span| family.contains(&span));
        state.touches |= member;
        state.safe &= member && problem.is_none();
        state.complete &= span.is_some() && problem.is_none();
        state.problem = state.problem.or(problem);
    }
    state.complete &= seen;
    state
}
