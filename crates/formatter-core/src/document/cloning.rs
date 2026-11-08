use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::document::{Align, Document, Group, IfBreak, IndentIfBreak};

/// Recursively clones a `Vec` of `Document`s into the given arena.
pub fn clone_vec_in_arena<'arena, A>(
    arena: &'arena A,
    source_vec: &Vec<'arena, Document<'arena, A>, A>,
) -> Vec<'arena, Document<'arena, A>, A>
where
    A: Arena,
{
    let mut new_vec = Vec::with_capacity_in(source_vec.len(), arena);
    new_vec.extend(source_vec.iter().map(|d| clone_in_arena(arena, d)));
    new_vec
}

/// Copy a document tree into an arena.
pub fn clone_in_arena<'arena, A>(arena: &'arena A, document: &Document<'arena, A>) -> Document<'arena, A>
where
    A: Arena,
{
    match document {
        Document::String(s) => Document::String(s),
        Document::RangeStart => Document::RangeStart,
        Document::RangeEnd => Document::RangeEnd,
        Document::Line(line) => Document::Line(*line),
        Document::Trim(trim) => Document::Trim(*trim),
        Document::Space(space) => Document::Space(*space),
        Document::Column(column) => Document::Column(*column),
        Document::LineSuffixBoundary => Document::LineSuffixBoundary,
        Document::BreakParent => Document::BreakParent,
        Document::DoNotTrim => Document::DoNotTrim,

        Document::Array(docs) => Document::Array(clone_vec_in_arena(arena, docs)),
        Document::Indent(docs) => Document::Indent(clone_vec_in_arena(arena, docs)),
        Document::LineSuffix(docs) => Document::LineSuffix(clone_vec_in_arena(arena, docs)),

        Document::IndentIfBreak(s) => Document::IndentIfBreak(IndentIfBreak {
            contents: clone_vec_in_arena(arena, &s.contents),
            group_id: s.group_id,
        }),
        Document::Group(g) => Document::Group(Group {
            contents: clone_vec_in_arena(arena, &g.contents),
            break_mode: g.break_mode.clone(),
            expanded_states: g.expanded_states.as_ref().map(|v| clone_vec_in_arena(arena, v)),
            id: g.id,
        }),
        Document::Align(a) => {
            Document::Align(Align { alignment: a.alignment, contents: clone_vec_in_arena(arena, &a.contents) })
        }

        Document::IfBreak(ib) => Document::IfBreak(IfBreak {
            break_contents: arena.alloc(clone_in_arena(arena, ib.break_contents)),
            flat_content: arena.alloc(clone_in_arena(arena, ib.flat_content)),
            group_id: ib.group_id,
        }),
    }
}
