use mago_allocator::Arena;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::operator::UnaryPrefixOperator;
use mago_hir::ir::statement::Foreach;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayFlag;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::ListFlag;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_foreach(
        &mut self,
        span: Span,
        foreach: &'source Foreach<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let iterator = self.infer_expression(foreach.expression)?;
        let (always_enters, key_type, value_type) = self.iterable_parameters(iterator.meta);

        let key = match foreach.key {
            Some(key) => Some(&*self.arena.alloc(self.bind_target(key, key_type)?)),
            None => None,
        };

        let value_target = match &foreach.value.kind {
            ExpressionKind::UnaryPrefix(prefix) if matches!(prefix.operator, UnaryPrefixOperator::Reference) => {
                prefix.operand
            }
            _ => foreach.value,
        };
        let value = self.bind_target(value_target, value_type)?;

        let outcome = self.analyze_loop(&[], &[], foreach.statement, false, always_enters, false)?;

        let node = Foreach {
            span: foreach.span,
            expression: self.arena.alloc(iterator),
            key,
            value: self.arena.alloc(value),
            statement: outcome.body,
        };

        Ok(Statement {
            meta: Flow { reachable: outcome.reachable, exit: outcome.exit },
            span,
            kind: StatementKind::Foreach(self.arena.alloc(node)),
        })
    }

    /// The `(always-enters, key, value)` an iterable yields. `always-enters` holds
    /// only when every part of the type is statically non-empty. Arrays, lists and
    /// `iterable<K, V>` are read precisely; anything else (objects, `mixed`) widens
    /// the key and value to `mixed`.
    fn iterable_parameters(&mut self, ty: Type<'arena>) -> (bool, Type<'arena>, Type<'arena>) {
        if ty.atoms.is_empty() {
            return (false, TYPE_NEVER, TYPE_NEVER);
        }

        let mut keys = self.ty.scratch_vec::<Atom<'arena>>();
        let mut values = self.ty.scratch_vec::<Atom<'arena>>();
        let mut always_enters = true;
        let mut imprecise = false;
        let mut key_imprecise = false;

        for atom in ty.atoms {
            match atom {
                Atom::Array(array) => {
                    if !array.flags.contains(ArrayFlag::NonEmpty) {
                        always_enters = false;
                    }
                    if let Some(items) = array.known_items {
                        for item in items {
                            match item.key {
                                ArrayKey::Int(value) => keys.push(Atom::Int(IntAtom::Literal(value))),
                                ArrayKey::String(bytes) => {
                                    let string = self.ty.string_literal_type(bytes);
                                    keys.extend_from_slice(string.atoms);
                                }
                                ArrayKey::Const { .. } => key_imprecise = true,
                            }
                            values.extend_from_slice(item.value.atoms);
                        }
                    }
                    if let Some(key_param) = array.key_param {
                        keys.extend_from_slice(key_param.atoms);
                    }
                    if let Some(value_param) = array.value_param {
                        values.extend_from_slice(value_param.atoms);
                    }
                }
                Atom::List(list) => {
                    if !list.flags.contains(ListFlag::NonEmpty) && list.known_count.is_none() {
                        always_enters = false;
                    }
                    keys.push(Atom::Int(IntAtom::Unspecified));
                    values.extend_from_slice(list.element_type.atoms);
                    if let Some(elements) = list.known_elements {
                        for element in elements {
                            values.extend_from_slice(element.value.atoms);
                        }
                    }
                }
                Atom::Iterable(iterable) => {
                    always_enters = false;
                    keys.extend_from_slice(iterable.key_type.atoms);
                    values.extend_from_slice(iterable.value_type.atoms);
                }
                Atom::Null | Atom::False => always_enters = false,
                _ => {
                    always_enters = false;
                    imprecise = true;
                }
            }
        }

        if imprecise {
            return (always_enters, TYPE_MIXED, TYPE_MIXED);
        }

        let key = if key_imprecise {
            TYPE_MIXED
        } else if keys.is_empty() {
            TYPE_NEVER
        } else {
            self.ty.union_of(&keys)
        };
        let value = if values.is_empty() { TYPE_NEVER } else { self.ty.union_of(&values) };

        (always_enters, key, value)
    }
}
