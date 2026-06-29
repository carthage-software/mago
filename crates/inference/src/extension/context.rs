use mago_allocator::Arena;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::float::LiteralFloat;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::well_known::TYPE_FLOAT;
use ordered_float::OrderedFloat;

/// What an extension may read and build while refining an expression. Holds the
/// real [`TypeBuilder`] directly (no trait object), so the type-construction
/// helpers monomorphize.
pub struct ExtensionContext<'ctx, 'source, 'arena, A: Arena> {
    builder: &'ctx mut TypeBuilder<'source, 'arena, A, A>,
    symbols: &'ctx SymbolTable<'arena, A>,
    namespace: &'ctx [u8],
}

impl<'ctx, 'source, 'arena, A: Arena> ExtensionContext<'ctx, 'source, 'arena, A> {
    pub(crate) fn new(
        builder: &'ctx mut TypeBuilder<'source, 'arena, A, A>,
        symbols: &'ctx SymbolTable<'arena, A>,
        namespace: &'ctx [u8],
    ) -> Self {
        Self { builder, symbols, namespace }
    }

    /// The union of the given atoms.
    pub fn union(&mut self, atoms: &[Atom<'arena>]) -> Type<'arena> {
        self.builder.union_of(atoms)
    }

    /// A literal integer type.
    pub fn int(&mut self, value: i64) -> Type<'arena> {
        self.builder.union_of(&[Atom::Int(IntAtom::Literal(value))])
    }

    /// A literal float type; non-finite values widen to `float`.
    pub fn float(&mut self, value: f64) -> Type<'arena> {
        if !value.is_finite() {
            return TYPE_FLOAT;
        }

        self.builder.union_of(&[Atom::Float(FloatAtom::Literal(LiteralFloat(OrderedFloat(value))))])
    }

    /// The symbol table for the program under inference.
    #[must_use]
    pub fn symbols(&self) -> &SymbolTable<'arena, A> {
        self.symbols
    }

    /// The namespace the current expression is in (empty for the global namespace).
    #[must_use]
    pub fn namespace(&self) -> &[u8] {
        self.namespace
    }
}
