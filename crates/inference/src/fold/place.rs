use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::AccessKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::var::Var;

use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'arena, A, S, E> InferenceFolder<'_, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn place_id(
        &mut self,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Option<Var<'arena>> {
        match &expression.kind {
            ExpressionKind::Parenthesized(inner) => self.place_id(inner),
            ExpressionKind::Variable(Variable::Direct(direct)) => Some(Var::new(direct.name)),
            ExpressionKind::Access(access) => match access.kind {
                AccessKind::Array(base, index) => self.array_place_id(base, index.meta),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn array_place_id(
        &mut self,
        base: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        index: Type<'arena>,
    ) -> Option<Var<'arena>> {
        let base_id = self.place_id(base)?;
        let key = self.array_key_of(index)?;

        let mut bytes = Vec::new_in(self.arena);
        bytes.extend_from_slice(base_id.as_bytes());
        bytes.push(b'[');
        match key {
            ArrayKey::Int(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            ArrayKey::String(value) => {
                bytes.push(b'\'');
                bytes.extend_from_slice(value);
                bytes.push(b'\'');
            }
            ArrayKey::Const { .. } => return None,
        }
        bytes.push(b']');

        Some(Var::new(bytes.leak()))
    }
}
