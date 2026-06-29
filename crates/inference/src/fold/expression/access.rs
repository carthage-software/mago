use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Access;
use mago_hir::ir::expression::AccessKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayAtom;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::ty::well_known::TYPE_NULL;
use mago_span::Span;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::reconciler::meet_with;
use crate::semantics::collect_closed_array;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_access(
        &mut self,
        span: Span,
        access: &'source Access<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        match access.kind {
            AccessKind::Array(array, index) => {
                let array = self.infer_expression(array)?;
                let index = self.infer_expression(index)?;
                let shape = self.array_element_type(array.meta, index.meta);
                let meta = match self.array_place_id(&array, index.meta) {
                    Some(place) => self.environment.lookup(place).unwrap_or(shape),
                    None => shape,
                };

                let node = Access {
                    span: access.span,
                    kind: AccessKind::Array(self.arena.alloc(array), self.arena.alloc(index)),
                };

                Ok(Expression { meta, span, kind: ExpressionKind::Access(self.arena.alloc(node)) })
            }
            _ => Err(InferenceError::Unsupported { span, construct: "member access" }),
        }
    }

    pub(crate) fn array_element_type(&mut self, array: Type<'arena>, key: Type<'arena>) -> Type<'arena> {
        if let [Atom::Array(atom)] = array.atoms {
            return self.keyed_element_type(atom, key);
        }

        let mut items = Vec::new_in(self.source);
        if !collect_closed_array(array, &mut items) {
            return TYPE_MIXED;
        }

        let Some(key) = self.array_key_of(key) else {
            let mut atoms = Vec::new_in(self.source);
            for item in &items {
                atoms.extend_from_slice(item.value.atoms);
            }

            atoms.push(Atom::Null);

            return self.ty.union_of(&atoms);
        };

        match items.iter().find(|item| item.key == key) {
            Some(item) if item.optional => self.union(item.value, TYPE_NULL),
            Some(item) => item.value,
            None => TYPE_NULL,
        }
    }

    fn keyed_element_type(&mut self, atom: &ArrayAtom<'arena>, key: Type<'arena>) -> Type<'arena> {
        let literal_key = self.array_key_of(key);

        let mut atoms = Vec::new_in(self.source);
        let mut definitely_present = false;

        if let Some(known_items) = atom.known_items {
            for item in known_items {
                let item_key = self.array_key_type(item.key);
                if !meet_with(&mut self.ty, self.symbols, key, item_key).is_never() {
                    atoms.extend_from_slice(item.value.atoms);
                }

                if !item.optional && literal_key == Some(item.key) {
                    definitely_present = true;
                }
            }
        }

        if let (Some(rest_key), Some(rest_value)) = (atom.key_param, atom.value_param)
            && !meet_with(&mut self.ty, self.symbols, key, rest_key).is_never()
        {
            atoms.extend_from_slice(rest_value.atoms);
        }

        if !definitely_present {
            atoms.push(Atom::Null);
        }

        self.ty.union_of(&atoms)
    }

    fn array_key_type(&mut self, key: ArrayKey<'arena>) -> Type<'arena> {
        match key {
            ArrayKey::Int(value) => self.int_literal(value),
            ArrayKey::String(value) => self.literal_string(value),
            ArrayKey::Const { .. } => TYPE_MIXED,
        }
    }

    pub(crate) fn remove_array_key(&mut self, array: Type<'arena>, key: ArrayKey<'arena>) -> Type<'arena> {
        let mut items = Vec::new_in(self.source);
        if !collect_closed_array(array, &mut items) {
            return array;
        }

        let mut kept = Vec::new_in(self.source);
        for item in &items {
            if item.key != key {
                kept.push(*item);
            }
        }

        self.closed_array(&kept)
    }
}
