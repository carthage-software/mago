use mago_allocator::prelude::*;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::Program;
use mago_syntax::cst::Statement;
use mago_syntax::cst::Trivia;
use mago_syntax::walker::Walker;
use mago_syntax::walker::walk_class_like_member;
use mago_syntax::walker::walk_statement;

use crate::Collector;

/// Attaches scopes to pragmas by walking the CST.
///
/// This function initializes and runs the `ScopeAttachmentWalker` over the entire program.
/// The walker traverses the CST to identify the precise CST node (e.g., a statement, a member,
/// a function or a class) that each pragma applies to.
pub fn attach_pragma_scopes<'arena, A>(collector: &mut Collector<'_, 'arena, A>, program: &Program<'arena>)
where
    A: Arena,
{
    ScopeAttachmentWalker { trivia: program.trivia.as_slice() }.walk_program(program, collector);
}

/// A CST walker that attaches a `scope_span` to each pragma.
///
/// By walking the CST, it can determine the exact node a pragma is intended to affect, solving
/// ambiguity issues with simple line-based checks: without a scope, a pragma could only be
/// matched by line number, which silently made it apply to *any* issue further down the file.
struct ScopeAttachmentWalker<'arena, 'trivia> {
    /// Every trivia (whitespace and comments) of the program, ordered by span.
    ///
    /// Used to tell "only comments and whitespace separate the pragma from the node" apart from
    /// "there is code in between".
    trivia: &'trivia [Trivia<'arena>],
}

impl ScopeAttachmentWalker<'_, '_> {
    /// Attach a scope to any applicable unscoped pragmas.
    ///
    /// A pragma is considered applicable to a node when it precedes the node with nothing but
    /// whitespace and comments in between: stacked pragma comments, a `/** @var … */` docblock
    /// and the like are part of the node's leading trivia, not code standing between the two.
    fn assign_scope_for_node<A>(&self, node_span: &Span, collector: &mut Collector<'_, '_, A>)
    where
        A: Arena,
    {
        for pragma in &mut collector.pragmas {
            // Skip pragmas that already have a scope.
            if pragma.scope_span.is_some() {
                continue;
            }

            // An inline pragma trails code on its own line (`foo(); // @mago-expect …`). It can
            // never own the node that follows it — the line rule in the collector handles it.
            if !pragma.own_line {
                continue;
            }

            let is_applicable = if pragma.trivia_span.end <= node_span.start {
                let start = pragma.trivia_span.end_offset() as usize;
                let end = node_span.start_offset() as usize;
                let between = &collector.file.contents[start..end];

                between.trim_ascii().is_empty()
                    || only_whitespace_and_comments(collector.file.contents.as_ref(), self.trivia, start, end)
            } else {
                false
            };

            if is_applicable {
                // Start the scope at the pragma itself: issues may be reported inside the
                // comments that lead up to the node (a docblock type reference, for example),
                // and those are still covered by the pragma that sits above them.
                pragma.scope_span =
                    Some(Span { file_id: node_span.file_id, start: pragma.trivia_span.start, end: node_span.end });
            }
        }
    }
}

/// Returns `true` when `contents[start..end]` holds nothing but whitespace and comments.
fn only_whitespace_and_comments(contents: &[u8], trivia: &[Trivia<'_>], start: usize, end: usize) -> bool {
    let mut cursor = start;

    // `trivia` is ordered by span, so the first candidate is found by binary search.
    let from = trivia.partition_point(|trivia| (trivia.span.end_offset() as usize) <= start);
    for trivia in &trivia[from..] {
        let trivia_start = trivia.span.start_offset() as usize;
        if trivia_start >= end {
            break;
        }

        if trivia_start > cursor && !contents[cursor..trivia_start].trim_ascii().is_empty() {
            return false;
        }

        cursor = cursor.max(trivia.span.end_offset() as usize);
    }

    cursor >= end || contents[cursor..end].trim_ascii().is_empty()
}

impl<'arena, A> Walker<'_, 'arena, Collector<'_, 'arena, A>> for ScopeAttachmentWalker<'arena, '_>
where
    A: Arena,
{
    /// Visits a statement and attaches scopes for any applicable pragmas.
    ///
    /// Every statement is a potential scope, not just declarations: a pragma placed above a
    /// plain statement covers that statement only, so an issue belonging to a *different*
    /// statement can never be suppressed by it.
    fn walk_statement(&self, statement: &Statement<'arena>, collector: &mut Collector<'_, 'arena, A>) {
        let span = statement.span();

        self.assign_scope_for_node(&span, collector);

        if collector.pragmas.iter().any(|p| p.scope_span.is_none() && span.contains(&p.span)) {
            walk_statement(self, statement, collector);
        }
    }

    /// Visits a class-like member and attaches scopes for any applicable pragmas.
    fn walk_class_like_member(&self, member: &ClassLikeMember<'arena>, collector: &mut Collector<'_, 'arena, A>) {
        let span = member.span();

        self.assign_scope_for_node(&span, collector);

        if collector.pragmas.iter().any(|p| p.scope_span.is_none() && span.contains(&p.span)) {
            walk_class_like_member(self, member, collector);
        }
    }
}
