use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::statement::For;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_for(
        &mut self,
        span: Span,
        for_loop: &'source For<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut initializations = Vec::new_in(self.arena);
        for initialization in for_loop.initializations {
            let typed = self.infer_expression(initialization)?;
            initializations.push(typed);
        }

        // No condition means nothing can end the loop, and the body always enters.
        let infinite = for_loop.conditions.is_empty();
        let always_enters = for_loop.conditions.is_empty();

        let conditions: std::vec::Vec<_> = for_loop.conditions.iter().collect();
        let increments: std::vec::Vec<_> = for_loop.increments.iter().collect();
        let outcome =
            self.analyze_loop(&conditions, &increments, for_loop.statement, false, always_enters, infinite)?;

        let node = For {
            span: for_loop.span,
            initializations: initializations.leak(),
            conditions: outcome.conditions,
            increments: outcome.increments,
            statement: outcome.body,
        };

        Ok(Statement {
            meta: Flow { reachable: outcome.reachable, exit: outcome.exit },
            span,
            kind: StatementKind::For(self.arena.alloc(node)),
        })
    }
}
