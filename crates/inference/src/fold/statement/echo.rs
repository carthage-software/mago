use mago_allocator::Arena;
use mago_allocator::vec::Vec;
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
    pub(crate) fn infer_echo(
        &mut self,
        span: Span,
        terminator: Option<Terminator>,
        expressions: &'source [Expression<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut items = Vec::new_in(self.arena);
        let mut diverges = false;
        for expression in expressions {
            let typed = self.infer_expression(expression)?;
            diverges |= typed.meta.is_never();
            items.push(typed);
        }

        let exit = if diverges { ControlFlow::Diverge } else { ControlFlow::Fallthrough };

        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit },
            span,
            kind: StatementKind::Echo(items.leak()),
            terminator,
        })
    }
}
