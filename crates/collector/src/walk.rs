use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;
use mago_syntax::walker::*;

use crate::Collector;

/// Attaches scopes to pragmas by walking the AST.
///
/// This function initializes and runs the `ScopeAttachmentWalker` over the entire program.
/// The walker traverses the AST to identify the precise AST node (e.g., a function or class)
/// that each pragma applies to.
pub fn attach_pragma_scopes<'arena>(collector: &mut Collector<'_, 'arena>, program: &Program<'arena>) {
    ScopeAttachmentWalker.walk_program(program, collector);
}

/// An AST walker that attaches a `scope_span` to each pragma.
///
/// By walking the AST, it can determine the exact declaration a pragma is intended to affect,
/// solving ambiguity issues with simple line-based checks.
struct ScopeAttachmentWalker;

impl ScopeAttachmentWalker {
    /// Attach a scope to any applicable unscoped pragmas.
    ///
    /// A pragma is considered applicable to a node if it immediately precedes
    /// the node's span with only whitespace in between. This method iterates through all
    /// unscoped pragmas and updates the first one that meets this criterion.
    fn assign_scope_for_node(&self, node_span: &Span, collector: &mut Collector<'_, '_>) {
        for pragma in collector.pragmas.iter_mut() {
            // Skip pragmas that already have a scope.
            if pragma.scope_span.is_some() {
                continue;
            }

            let is_applicable = if pragma.trivia_span.end <= node_span.start {
                let between =
                    &collector.file.contents[pragma.trivia_span.end.offset as usize..node_span.start.offset as usize];

                between.trim().is_empty()
            } else {
                false
            };

            if is_applicable {
                pragma.scope_span = Some(*node_span);
            }
        }
    }
}

impl<'ast, 'arena> Walker<'ast, 'arena, Collector<'_, 'arena>> for ScopeAttachmentWalker {
    /// Visits a statement and attaches scopes for any applicable pragmas.
    fn walk_statement(&self, statement: &Statement<'arena>, collector: &mut Collector<'_, 'arena>) {
        let span = statement.span();

        if statement.is_declaration() {
            self.assign_scope_for_node(&span, collector);
        }

        if collector.pragmas.iter().any(|p| p.scope_span.is_none() && span.contains(&p.span)) {
            walk_statement(self, statement, collector);
        }
    }

    /// Visits a class-like member and attaches scopes for any applicable pragmas.
    fn walk_class_like_member(&self, member: &ClassLikeMember<'arena>, collector: &mut Collector<'_, 'arena>) {
        let span = member.span();

        self.assign_scope_for_node(&span, collector);

        if collector.pragmas.iter().any(|p| p.scope_span.is_none() && span.contains(&p.span)) {
            walk_class_like_member(self, member, collector);
        }
    }
}
