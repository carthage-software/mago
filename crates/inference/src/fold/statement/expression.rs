use mago_allocator::Arena;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_expression_statement(
        &mut self,
        span: Span,
        expression: &'source Expression<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let inferred_expression = self.infer_expression(expression)?;

        let exit = if inferred_expression.meta.is_never() { ControlFlow::Diverge } else { ControlFlow::Fallthrough };

        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit },
            span,
            kind: StatementKind::Expression(self.arena.alloc(inferred_expression)),
        })
    }
}
