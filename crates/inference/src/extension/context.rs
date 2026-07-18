use mago_allocator::Arena;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::TypeBuilder;

pub struct ExtensionContext<'ctx, 'source, 'arena, A: Arena> {
    pub ty: &'ctx mut TypeBuilder<'source, 'arena, A, A>,
    pub symbols: &'ctx SymbolTable<'arena, A>,
    pub namespace: &'ctx [u8],
}

impl<'ctx, 'source, 'arena, A: Arena> ExtensionContext<'ctx, 'source, 'arena, A> {
    pub(crate) fn new(
        ty: &'ctx mut TypeBuilder<'source, 'arena, A, A>,
        symbols: &'ctx SymbolTable<'arena, A>,
        namespace: &'ctx [u8],
    ) -> Self {
        Self { ty, symbols, namespace }
    }
}
