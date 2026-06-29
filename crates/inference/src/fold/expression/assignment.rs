use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::ArrayElement;
use mago_hir::ir::expression::ArrayElementKind;
use mago_hir::ir::expression::Assignment;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::annotation::Annotation;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::lower_type_annotation;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::semantics::collect_closed_array;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_assignment(
        &mut self,
        span: Span,
        assignment: &'source Assignment<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let value = self.infer_expression(assignment.right)?;

        let value_meta = match assignment.operator {
            None => value.meta,
            Some(_) => return Err(InferenceError::Unsupported { span, construct: "compound assignment" }),
        };

        let target = self.bind_target(assignment.left, value_meta)?;
        let meta = target.meta;

        let assignment = Assignment {
            span: assignment.span,
            left: self.arena.alloc(target),
            operator: assignment.operator,
            right: self.arena.alloc(value),
        };

        Ok(Expression { meta, span, kind: ExpressionKind::Assignment(self.arena.alloc(assignment)) })
    }

    fn bind_target(
        &mut self,
        target: &'source Expression<'source, SymbolId, S, E>,
        ty: Type<'arena>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let node = match &target.kind {
            ExpressionKind::Variable(variable) => {
                let variable = match variable {
                    Variable::Direct(direct) => {
                        let direct = direct.copy_into(self.arena);
                        self.environment.set(Var::new(direct.name), ty);
                        self.environment.invalidate_rooted_in(Var::new(direct.name));

                        Variable::Direct(direct)
                    }
                    Variable::Indirect(expression) => {
                        let expression = self.infer_expression(expression)?;

                        Variable::Indirect(self.arena.alloc(expression))
                    }
                    Variable::Nested(_) => {
                        return Err(InferenceError::Unsupported {
                            span: target.span,
                            construct: "a variable-variable assignment target",
                        });
                    }
                };

                Expression { meta: ty, span: target.span, kind: ExpressionKind::Variable(variable) }
            }
            ExpressionKind::Annotation(annotation) => {
                let type_annotation = annotation.annotation.type_annotation.copy_into(self.arena);
                let annotated = lower_type_annotation(&mut self.ty, &type_annotation).unwrap_or(ty);

                let inner = self.bind_target(annotation.expression, annotated)?;

                let variable_annotation = annotation.annotation.copy_into(self.arena);
                let node = Annotation {
                    annotation: self.arena.alloc(variable_annotation),
                    expression: self.arena.alloc(inner),
                };

                Expression {
                    meta: annotated,
                    span: target.span,
                    kind: ExpressionKind::Annotation(self.arena.alloc(node)),
                }
            }
            ExpressionKind::Array(elements) => {
                let elements = self.bind_destructure(elements, ty)?;

                Expression { meta: ty, span: target.span, kind: ExpressionKind::Array(elements) }
            }
            ExpressionKind::List(elements) => {
                let elements = self.bind_destructure(elements, ty)?;

                Expression { meta: ty, span: target.span, kind: ExpressionKind::List(elements) }
            }
            _ => return Err(InferenceError::Unsupported { span: target.span, construct: "this assignment target" }),
        };

        Ok(node)
    }

    /// Destructures `ty` into a list/array assignment target, binding each
    /// element target to the type at its position (`[$a, $b] = ...`) or explicit
    /// key (`['k' => $v] = ...`). A missing element is skipped, and a position
    /// not present in the source type binds `mixed`.
    fn bind_destructure(
        &mut self,
        elements: &'source Delimited<'source, ArrayElement<'source, SymbolId, S, E>>,
        ty: Type<'arena>,
    ) -> InferenceResult<Delimited<'arena, ArrayElement<'arena, SymbolId, Flow, Type<'arena>>>> {
        let mut items = Vec::new_in(self.source);
        collect_closed_array(ty, &mut items);

        let mut index = 0i64;
        let mut typed = Vec::new_in(self.arena);
        for element in elements.items {
            let kind = match element.kind {
                ArrayElementKind::Value(target) => {
                    let element_type = element_type_for(&items, ArrayKey::Int(index));
                    index += 1;

                    ArrayElementKind::Value(self.arena.alloc(self.bind_target(target, element_type)?))
                }
                ArrayElementKind::KeyValue(key, target) => {
                    let key = self.infer_expression(key)?;
                    let element_type =
                        self.array_key_of(key.meta).map_or(TYPE_MIXED, |key| element_type_for(&items, key));

                    ArrayElementKind::KeyValue(
                        self.arena.alloc(key),
                        self.arena.alloc(self.bind_target(target, element_type)?),
                    )
                }
                ArrayElementKind::Missing => {
                    index += 1;

                    ArrayElementKind::Missing
                }
                ArrayElementKind::Variadic(target) => {
                    ArrayElementKind::Variadic(self.arena.alloc(self.bind_target(target, TYPE_MIXED)?))
                }
            };

            typed.push(ArrayElement { span: element.span, kind });
        }

        Ok(Delimited { span: elements.span, items: typed.leak() })
    }
}

/// The value type stored at `key` in a collected closed-shape entry list, or
/// `mixed` when the key is absent (an open or unknown source shape).
fn element_type_for<'arena>(items: &[KnownItem<'arena>], key: ArrayKey<'arena>) -> Type<'arena> {
    items.iter().find(|item| item.key == key).map_or(TYPE_MIXED, |item| item.value)
}
