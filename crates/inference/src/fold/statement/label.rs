use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::name::Name;
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
    /// TODO(azjezz): How do we support `goto` statements? Not sure.
    pub(crate) fn infer_label(
        &self,
        span: Span,
        label: Name<'source>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::Label(label.copy_into(self.arena)),
        })
    }
}
