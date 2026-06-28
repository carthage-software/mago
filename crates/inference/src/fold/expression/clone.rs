use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_span::Span;

use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_clone(
        &mut self,
        span: Span,
        operand: &'source Expression<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let operand = self.infer_expression(operand);

        let meta = if operand.meta.is_never() { TYPE_NEVER } else { self.clone_type(operand.meta) };

        Expression { meta, span, kind: ExpressionKind::Clone(self.arena.alloc(operand)) }
    }

    /// `clone` evaluates to its operand's type, keeping only the cloneable atoms:
    /// objects (an enum is not cloneable) and `mixed` (which might be an object).
    /// Non-object operands are a fatal error, so they contribute nothing and an
    /// all-non-object operand becomes `never`.
    fn clone_type(&mut self, ty: Type<'arena>) -> Type<'arena> {
        let mut atoms = Vec::new_in(self.source);
        for atom in ty.atoms {
            if is_cloneable(atom) {
                atoms.push(*atom);
            }
        }

        self.ty.union_of(&atoms)
    }
}

fn is_cloneable(atom: &Atom<'_>) -> bool {
    matches!(
        atom.kind(),
        AtomKind::Object
            | AtomKind::ObjectShape
            | AtomKind::HasMethod
            | AtomKind::HasProperty
            | AtomKind::ObjectAny
            | AtomKind::Mixed
    )
}
