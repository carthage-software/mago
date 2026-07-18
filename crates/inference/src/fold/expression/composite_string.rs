use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::CompositeString;
use mago_hir::ir::expression::CompositeStringKind;
use mago_hir::ir::expression::CompositeStringPart;
use mago_hir::ir::expression::CompositeStringPartKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::FALSE;
use mago_oracle::ty::well_known::NON_EMPTY_STRING;
use mago_oracle::ty::well_known::NULL;
use mago_oracle::ty::well_known::STRING;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_oracle::ty::well_known::TYPE_STRING;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::semantics::append_string;

type TypedPart<'arena> = CompositeStringPart<'arena, SymbolId, Flow, Type<'arena>>;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_composite_string(
        &mut self,
        span: Span,
        composite: &'source CompositeString<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        if composite.kind == CompositeStringKind::ShellExecute {
            return self.infer_shell_execute(span, composite);
        }

        let mut rebuilt = Vec::new_in(self.arena);
        let mut bytes = Vec::new_in(self.source);
        let mut foldable = true;
        let mut non_empty = false;
        let mut has_never = false;

        for part in composite.parts {
            let kind = match &part.kind {
                CompositeStringPartKind::Literal(literal) => {
                    if !literal.is_empty() {
                        non_empty = true;
                    }
                    if foldable {
                        bytes.extend_from_slice(literal);
                    }

                    CompositeStringPartKind::Literal(self.arena.alloc_slice_copy(literal))
                }
                CompositeStringPartKind::Expression(expression)
                | CompositeStringPartKind::BracedExpression(expression) => {
                    let expression = self.infer_expression(expression)?;
                    has_never |= expression.meta.is_never();
                    if foldable && !append_string(&mut bytes, expression.meta) {
                        foldable = false;
                    }

                    CompositeStringPartKind::Expression(self.arena.alloc(expression))
                }
            };

            rebuilt.push(CompositeStringPart { span: part.span, kind });
        }

        let meta = if has_never {
            TYPE_NEVER
        } else if foldable {
            self.ty.string_literal_type(&bytes)
        } else if non_empty {
            self.ty.union_of(&[NON_EMPTY_STRING])
        } else {
            TYPE_STRING
        };

        Ok(Expression {
            meta,
            span,
            kind: ExpressionKind::CompositeString(self.arena.alloc(CompositeString {
                span: composite.span,
                kind: composite.kind,
                parts: rebuilt.leak(),
            })),
        })
    }

    fn infer_shell_execute(
        &mut self,
        span: Span,
        composite: &'source CompositeString<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let (parts, has_never) = self.rebuild_string_parts(composite.parts)?;

        let meta = if has_never { TYPE_NEVER } else { self.ty.union_of(&[STRING, FALSE, NULL]) };

        Ok(Expression {
            meta,
            span,
            kind: ExpressionKind::CompositeString(self.arena.alloc(CompositeString {
                span: composite.span,
                kind: composite.kind,
                parts,
            })),
        })
    }

    fn rebuild_string_parts(
        &mut self,
        parts: &'source [CompositeStringPart<'source, SymbolId, S, E>],
    ) -> InferenceResult<(&'arena [TypedPart<'arena>], bool)> {
        let mut rebuilt = Vec::new_in(self.arena);
        let mut has_never = false;

        for part in parts {
            let kind = match &part.kind {
                CompositeStringPartKind::Literal(literal) => {
                    CompositeStringPartKind::Literal(self.arena.alloc_slice_copy(literal))
                }
                CompositeStringPartKind::Expression(expression)
                | CompositeStringPartKind::BracedExpression(expression) => {
                    let expression = self.infer_expression(expression)?;
                    has_never |= expression.meta.is_never();

                    CompositeStringPartKind::Expression(self.arena.alloc(expression))
                }
            };

            rebuilt.push(CompositeStringPart { span: part.span, kind });
        }

        Ok((rebuilt.leak(), has_never))
    }
}
