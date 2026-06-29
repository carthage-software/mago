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
    pub(crate) fn infer_continue(
        &mut self,
        span: Span,
        level: Option<&'source Expression<'source, SymbolId, S, E>>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let reachable = self.reachable;
        let (depth, level) = self.infer_loop_level(level)?;
        let exit = match depth {
            Some(depth) => {
                if reachable {
                    self.record_loop_exit(depth, false);
                }

                ControlFlow::Continue(depth)
            }
            None => ControlFlow::Diverge,
        };

        Ok(Statement { meta: Flow { reachable, exit }, span, kind: StatementKind::Continue(level) })
    }
}
