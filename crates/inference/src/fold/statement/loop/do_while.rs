use mago_allocator::Arena;
use mago_hir::ir::statement::DoWhile;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Terminator;
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
    pub(crate) fn infer_do_while(
        &mut self,
        span: Span,
        terminator: Option<Terminator>,
        do_while: &'source DoWhile<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let infinite = is_truthy_literal(do_while.condition);
        // The body always runs once before the condition is ever evaluated.
        let outcome = self.analyze_loop(&[do_while.condition], &[], do_while.statement, true, true, infinite)?;

        let node = DoWhile { span: do_while.span, statement: outcome.body, condition: &outcome.conditions[0] };

        Ok(Statement {
            meta: Flow { reachable: outcome.reachable, exit: outcome.exit },
            span,
            kind: StatementKind::DoWhile(self.arena.alloc(node)),
            terminator,
        })
    }
}
