use mago_allocator::Arena;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::Yield;
use mago_hir::ir::expression::YieldKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_span::Span;

use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_yield(
        &mut self,
        span: Span,
        yield_expression: &'source Yield<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let kind = match yield_expression.kind {
            YieldKind::Nothing => YieldKind::Nothing,
            YieldKind::Expression(value) => {
                let value = self.infer_expression(value);

                YieldKind::Expression(self.arena.alloc(value))
            }
            YieldKind::Pair(key, value) => {
                let key = self.infer_expression(key);
                let value = self.infer_expression(value);

                YieldKind::Pair(self.arena.alloc(key), self.arena.alloc(value))
            }
            YieldKind::From(value) => {
                let value = self.infer_expression(value);

                YieldKind::From(self.arena.alloc(value))
            }
        };

        let node = Yield { span: yield_expression.span, kind };

        // TODO(azjezz): this should be the `Send` type parameter of the enclosing generator,
        // but we don't track that yet. `mixed` will do FOR NOW!
        Expression { meta: TYPE_MIXED, span, kind: ExpressionKind::Yield(self.arena.alloc(node)) }
    }
}
