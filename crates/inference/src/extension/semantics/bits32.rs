use mago_allocator::Arena;
use mago_hir::ir::expression::Expression;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::float::LiteralFloat;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use ordered_float::OrderedFloat;

use crate::extension::ExtensionContext;
use crate::extension::ExtensionInference;
use crate::flow::Flow;

/// Forces 32-bit integer semantics.
#[derive(Debug, Default, Clone, Copy)]
pub struct Bits32Extension;

impl<A: Arena> ExtensionInference<A> for Bits32Extension {
    fn infer<'ctx, 'source, 'arena>(
        &self,
        context: &mut ExtensionContext<'ctx, 'source, 'arena, A>,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Option<Type<'arena>> {
        if let [Atom::Int(IntAtom::Literal(value))] = expression.meta.atoms {
            return i32::try_from(*value).is_err().then(|| context.float(*value as f64));
        }

        if !expression.meta.atoms.iter().any(overflows_i32) {
            return None;
        }

        let mut atoms = std::vec::Vec::with_capacity(expression.meta.atoms.len());
        for atom in expression.meta.atoms {
            match atom {
                Atom::Int(IntAtom::Literal(value)) if i32::try_from(*value).is_err() => {
                    atoms.push(Atom::Float(FloatAtom::Literal(LiteralFloat(OrderedFloat(*value as f64)))));
                }
                other => atoms.push(*other),
            }
        }

        Some(context.union(&atoms))
    }
}

fn overflows_i32(atom: &Atom<'_>) -> bool {
    matches!(atom, Atom::Int(IntAtom::Literal(value)) if i32::try_from(*value).is_err())
}
