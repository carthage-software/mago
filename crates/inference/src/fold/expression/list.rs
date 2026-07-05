use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::ArrayElement;
use mago_hir::ir::expression::ArrayElementKind;
use mago_hir::ir::expression::ArrayLike;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_list(
        &mut self,
        span: Span,
        array: &ArrayLike<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut items = Vec::new_in(self.arena);
        for element in array.elements.items {
            let kind = match element.kind {
                ArrayElementKind::Value(value) => {
                    let value = self.infer_expression(value)?;

                    ArrayElementKind::Value(self.arena.alloc(value))
                }
                ArrayElementKind::KeyValue(key, value) => {
                    let key = self.infer_expression(key)?;
                    let value = self.infer_expression(value)?;

                    ArrayElementKind::KeyValue(self.arena.alloc(key), self.arena.alloc(value))
                }
                ArrayElementKind::Variadic(value) => {
                    let value = self.infer_expression(value)?;

                    ArrayElementKind::Variadic(self.arena.alloc(value))
                }
                ArrayElementKind::Missing => ArrayElementKind::Missing,
            };

            items.push(ArrayElement { span: element.span, kind });
        }

        Ok(Expression {
            meta: TYPE_NEVER,
            span,
            kind: ExpressionKind::ArrayLike(ArrayLike {
                span: array.span,
                kind: array.kind,
                elements: Delimited { span: array.elements.span, items: items.leak() },
            }),
        })
    }
}
