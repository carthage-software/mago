use mago_allocator::Arena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::Type;

use crate::flow::Flow;
use crate::fold::InferenceFolder;

pub mod flow;
pub mod reconciler;
pub mod tdd;

mod fold;
mod semantics;

#[derive(Debug)]
pub struct Inference<'source, 'arena, A>
where
    A: Arena,
{
    source: &'source A,
    arena: &'arena A,
}

impl<'source, 'arena, A> Inference<'source, 'arena, A>
where
    A: Arena,
{
    pub fn new(source: &'source A, arena: &'arena A) -> Self {
        Self { source, arena }
    }

    pub fn infer<'symbols, S, E>(
        &mut self,
        symbols: &'symbols SymbolTable<'arena, A>,
        file: &File,
        ir: IR<'source, SymbolId, S, E>,
    ) -> IR<'arena, SymbolId, Flow, Type<'arena>> {
        InferenceFolder::new(self.source, self.arena, symbols, file).infer_ir(ir)
    }
}
