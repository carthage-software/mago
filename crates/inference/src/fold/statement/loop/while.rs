use mago_allocator::Arena;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::While;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::fold::statement::r#loop::is_truthy_literal;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_while(
        &mut self,
        span: Span,
        while_loop: &'source While<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let infinite = is_truthy_literal(while_loop.condition);
        let outcome =
            self.analyze_loop(&[while_loop.condition], &[], while_loop.statement, false, infinite, infinite)?;

        let node = While { span: while_loop.span, condition: &outcome.conditions[0], statement: outcome.body };

        Ok(Statement {
            meta: Flow { reachable: outcome.reachable, exit: outcome.exit },
            span,
            kind: StatementKind::While(self.arena.alloc(node)),
        })
    }
}
