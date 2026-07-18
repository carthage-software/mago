use mago_allocator::Arena;
use mago_allocator::copy::copy_slice_into;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Terminator;
use mago_hir::ir::statement::UseItem;
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
    pub(crate) fn infer_use(
        &self,
        span: Span,
        terminator: Option<Terminator>,
        items: &[UseItem<'source>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::Use(copy_slice_into(items, self.arena)),
            terminator,
        })
    }
}
