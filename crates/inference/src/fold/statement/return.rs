use mago_allocator::Arena;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Terminator;
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
    pub(crate) fn infer_return(
        &mut self,
        span: Span,
        terminator: Option<Terminator>,
        value: Option<&'source Expression<'source, SymbolId, S, E>>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let value = match value {
            Some(value) => {
                let value = self.infer_expression(value)?;

                Some(&*self.arena.alloc(value))
            }
            None => None,
        };

        let exit = match value {
            Some(value) if value.meta.is_never() => ControlFlow::Diverge,
            _ => ControlFlow::Return,
        };

        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit },
            span,
            kind: StatementKind::Return(value),
            terminator,
        })
    }
}
