use mago_allocator::Arena;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Terminator;
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
    pub(crate) fn infer_sequence(
        &mut self,
        span: Span,
        terminator: Option<Terminator>,
        statements: &'source [Statement<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let reachable = self.reachable;
        let (items, exit) = self.infer_block(statements)?;

        Ok(Statement { meta: Flow { reachable, exit }, span, kind: StatementKind::Sequence(items), terminator })
    }
}
