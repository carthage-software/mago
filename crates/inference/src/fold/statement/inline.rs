use mago_allocator::Arena;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'arena, A, S, E> InferenceFolder<'_, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_inline(
        &self,
        span: Span,
        content: &[u8],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::Inline(self.arena.alloc_slice_copy(content)),
        })
    }
}
