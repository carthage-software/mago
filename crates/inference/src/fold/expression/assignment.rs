use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_flags::U8Flags;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::Access;
use mago_hir::ir::expression::AccessKind;
use mago_hir::ir::expression::ArrayElement;
use mago_hir::ir::expression::ArrayElementKind;
use mago_hir::ir::expression::Assignment;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::annotation::Annotation;
use mago_hir::ir::expression::operator::AssignmentOperator;
use mago_hir::ir::expression::operator::BinaryOperator;
use mago_hir::ir::expression::selector::MemberSelector;
use mago_hir::ir::expression::selector::MemberSelectorKind;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::lower_type_annotation;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayAtom;
use mago_oracle::ty::atom::payload::array::ArrayFlag;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::atom::payload::array::ListFlag;
use mago_oracle::ty::join;
use mago_oracle::ty::well_known::TYPE_ARRAY_KEY;
use mago_oracle::ty::well_known::TYPE_INT;
use mago_oracle::ty::well_known::TYPE_INT_OR_STRING;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::fold::expression::array::push_entry;
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

        // `$target OP= $value` is `$target = $target OP $value`: the new value is the
        // binary operation against the target's current type, then bound back.
        let assigned = match assignment.operator {
            None => value.meta,
            Some(operator) => {
                let current = self.infer_expression(assignment.left)?.meta;

                self.binary_type(compound_operator(operator), current, value.meta)
            }
        };

        let target = self.bind_target(assignment.left, assigned)?;
        let meta = target.meta;

        let assignment = Assignment {
            span: assignment.span,
            left: self.arena.alloc(target),
            operator: assignment.operator,
            right: self.arena.alloc(value),
        };

        Ok(Expression { meta, span, kind: ExpressionKind::Assignment(self.arena.alloc(assignment)) })
    }

    pub(crate) fn bind_target(
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
                        self.bind_dynamic_variable(expression.meta, ty, target.span)?;

                        Variable::Indirect(self.arena.alloc(expression))
                    }
                    Variable::Nested(inner) => {
                        let (name, inner) = self.fold_variable(inner)?;
                        self.bind_dynamic_variable(name, ty, target.span)?;

                        Variable::Nested(self.arena.alloc(inner))
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
            ExpressionKind::ArrayAppend(base) => {
                let current = self.infer_expression(base)?.meta;
                let updated = self.array_after_write(current, WriteKey::Append, ty);
                let base = self.bind_target(base, updated)?;

                Expression { meta: ty, span: target.span, kind: ExpressionKind::ArrayAppend(self.arena.alloc(base)) }
            }
            ExpressionKind::Access(access) => match &access.kind {
                AccessKind::Array(base, index) => {
                    let index = self.infer_expression(index)?;
                    let current = self.infer_expression(base)?.meta;
                    let updated = match self.array_key_of(index.meta) {
                        Some(key) => self.array_after_write(current, WriteKey::Known(key), ty),
                        None => {
                            let key_type = self.array_key_contribution(index.meta);

                            self.array_after_write(current, WriteKey::Dynamic(key_type), ty)
                        }
                    };

                    let base = self.bind_target(base, updated)?;
                    let node = Access {
                        span: access.span,
                        kind: AccessKind::Array(self.arena.alloc(base), self.arena.alloc(index)),
                    };

                    Expression { meta: ty, span: target.span, kind: ExpressionKind::Access(self.arena.alloc(node)) }
                }
                AccessKind::Property(object, selector) => {
                    let (object, selector) = self.bind_object_property(object, selector, ty)?;
                    let node =
                        Access { span: access.span, kind: AccessKind::Property(self.arena.alloc(object), selector) };

                    Expression { meta: ty, span: target.span, kind: ExpressionKind::Access(self.arena.alloc(node)) }
                }
                AccessKind::NullsafeProperty(object, selector) => {
                    let (object, selector) = self.bind_object_property(object, selector, ty)?;
                    let node = Access {
                        span: access.span,
                        kind: AccessKind::NullsafeProperty(self.arena.alloc(object), selector),
                    };

                    Expression { meta: ty, span: target.span, kind: ExpressionKind::Access(self.arena.alloc(node)) }
                }
                AccessKind::StaticProperty(class, variable) => {
                    let class = self.infer_expression(class)?;
                    let variable = self.infer_variable_node(variable)?;
                    let node = Access {
                        span: access.span,
                        kind: AccessKind::StaticProperty(self.arena.alloc(class), variable),
                    };

                    Expression { meta: ty, span: target.span, kind: ExpressionKind::Access(self.arena.alloc(node)) }
                }
                _ => {
                    return Err(InferenceError::Unsupported { span: target.span, construct: "this assignment target" });
                }
            },
            ExpressionKind::Parenthesized(inner) => {
                let inner = self.bind_target(inner, ty)?;

                Expression {
                    meta: inner.meta,
                    span: target.span,
                    kind: ExpressionKind::Parenthesized(self.arena.alloc(inner)),
                }
            }
            _ => return Err(InferenceError::Unsupported { span: target.span, construct: "this assignment target" }),
        };

        Ok(node)
    }

    fn bind_object_property(
        &mut self,
        object: &'source Expression<'source, SymbolId, S, E>,
        selector: &MemberSelector<'source, SymbolId, S, E>,
        ty: Type<'arena>,
    ) -> InferenceResult<(
        Expression<'arena, SymbolId, Flow, Type<'arena>>,
        MemberSelector<'arena, SymbolId, Flow, Type<'arena>>,
    )> {
        let object = self.infer_expression(object)?;
        if let MemberSelectorKind::Name(name) = &selector.kind
            && let Some(place) = self.property_place_id(&object, name.value)
        {
            self.environment.set(place, ty);
        }

        let selector = self.infer_member_selector(selector)?;

        Ok((object, selector))
    }

    fn bind_dynamic_variable(&mut self, name_type: Type<'arena>, ty: Type<'arena>, span: Span) -> InferenceResult<()> {
        let Some(name) = self.resolved_variable_name(name_type) else {
            return Err(InferenceError::Unsupported { span, construct: "a variable-variable assignment target" });
        };

        self.environment.set(Var::new(name), ty);
        self.environment.invalidate_rooted_in(Var::new(name));

        Ok(())
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

    fn array_after_write(&mut self, current: Type<'arena>, key: WriteKey<'arena>, value: Type<'arena>) -> Type<'arena> {
        let (mut items, mut rest_key, mut rest_value, mut non_empty) = self.decompose_array(current);

        match key {
            WriteKey::Known(key) => push_entry(&mut items, key, value),
            WriteKey::Append if rest_key.is_none() => {
                let index = next_index(&items);
                push_entry(&mut items, ArrayKey::Int(index), value);
            }
            WriteKey::Append => {
                rest_key = Some(self.union_into(rest_key, TYPE_INT));
                rest_value = Some(self.union_into(rest_value, value));
                non_empty = true;
            }
            WriteKey::Dynamic(key_type) => {
                rest_key = Some(self.union_into(rest_key, key_type));
                rest_value = Some(self.union_into(rest_value, value));
            }
        }

        match (rest_key, rest_value) {
            (Some(key_param), Some(value_param)) => {
                items.sort_unstable_by(|left, right| left.key.cmp(&right.key));
                let known_items = (!items.is_empty()).then(|| self.ty.known_items(&items));
                let mut flags = U8Flags::empty();
                flags.set_value(ArrayFlag::NonEmpty, non_empty);
                let atom = self.ty.array(ArrayAtom {
                    key_param: Some(normalize_array_key(key_param)),
                    value_param: Some(value_param),
                    known_items,
                    flags,
                });

                self.ty.union_of(&[atom])
            }
            _ => self.closed_array(&items),
        }
    }

    fn decompose_array(
        &self,
        current: Type<'arena>,
    ) -> (Vec<'source, KnownItem<'arena>, A>, Option<Type<'arena>>, Option<Type<'arena>>, bool) {
        let mut items = Vec::new_in(self.source);
        let mut rest_key = None;
        let mut rest_value = None;
        let mut non_empty = false;

        match current.atoms {
            [Atom::Array(array)] => {
                if let Some(known) = array.known_items {
                    items.extend_from_slice(known);
                }
                rest_key = array.key_param;
                rest_value = array.value_param;
                non_empty = array.flags.contains(ArrayFlag::NonEmpty);
            }
            [Atom::List(list)] => {
                if let Some(known) = list.known_elements {
                    for element in known {
                        items.push(KnownItem {
                            key: ArrayKey::Int(i64::from(element.index)),
                            value: element.value,
                            optional: element.optional,
                        });
                    }
                }
                if !list.element_type.is_never() {
                    rest_key = Some(TYPE_INT);
                    rest_value = Some(list.element_type);
                }
                non_empty = list.flags.contains(ListFlag::NonEmpty);
            }
            _ => {}
        }

        (items, rest_key, rest_value, non_empty)
    }

    fn array_key_contribution(&mut self, index: Type<'arena>) -> Type<'arena> {
        let mut atoms = Vec::new_in(self.source);
        for atom in index.atoms {
            match atom {
                Atom::Int(_) | Atom::String(_) => atoms.push(*atom),
                _ => return TYPE_ARRAY_KEY,
            }
        }

        if atoms.is_empty() { TYPE_ARRAY_KEY } else { self.ty.union_of(&atoms) }
    }

    fn union_into(&mut self, current: Option<Type<'arena>>, extra: Type<'arena>) -> Type<'arena> {
        let Some(existing) = current else {
            return extra;
        };

        let mut atoms = Vec::new_in(self.source);
        atoms.extend_from_slice(existing.atoms);
        atoms.extend_from_slice(extra.atoms);

        let canonical = join::compute(&atoms, &mut self.ty);

        self.ty.union_of(&canonical)
    }
}

/// The value type stored at `key` in a collected closed-shape entry list, or
/// `mixed` when the key is absent (an open or unknown source shape).
fn element_type_for<'arena>(items: &[KnownItem<'arena>], key: ArrayKey<'arena>) -> Type<'arena> {
    items.iter().find(|item| item.key == key).map_or(TYPE_MIXED, |item| item.value)
}

enum WriteKey<'arena> {
    Known(ArrayKey<'arena>),
    Append,
    Dynamic(Type<'arena>),
}

fn normalize_array_key(key: Type<'_>) -> Type<'_> {
    if key.atoms == TYPE_INT_OR_STRING.atoms { TYPE_ARRAY_KEY } else { key }
}

fn next_index(items: &[KnownItem<'_>]) -> i64 {
    items
        .iter()
        .filter_map(|item| match item.key {
            ArrayKey::Int(index) => Some(index),
            _ => None,
        })
        .max()
        .map_or(0, |index| index + 1)
}

/// The binary operator a compound assignment desugars to (`+=` is `+`, `.=` is
/// `.`, `??=` is `??`).
fn compound_operator(operator: AssignmentOperator) -> BinaryOperator {
    match operator {
        AssignmentOperator::Addition => BinaryOperator::Addition,
        AssignmentOperator::Subtraction => BinaryOperator::Subtraction,
        AssignmentOperator::Multiplication => BinaryOperator::Multiplication,
        AssignmentOperator::Division => BinaryOperator::Division,
        AssignmentOperator::Modulo => BinaryOperator::Modulo,
        AssignmentOperator::Exponentiation => BinaryOperator::Exponentiation,
        AssignmentOperator::Concat => BinaryOperator::StringConcat,
        AssignmentOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
        AssignmentOperator::BitwiseOr => BinaryOperator::BitwiseOr,
        AssignmentOperator::BitwiseXor => BinaryOperator::BitwiseXor,
        AssignmentOperator::LeftShift => BinaryOperator::LeftShift,
        AssignmentOperator::RightShift => BinaryOperator::RightShift,
        AssignmentOperator::Coalesce => BinaryOperator::NullCoalesce,
    }
}
