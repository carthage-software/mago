use mago_allocator::Arena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::Type;

use crate::extension::Extensions;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

pub mod error;
pub mod extension;
pub mod flow;
pub mod reconciler;
pub mod tdd;

mod fold;
mod semantics;

pub use crate::error::InferenceError;
pub use crate::error::InferenceResult;

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

    /// Infers types for `ir` against `symbols`, returning the fully-typed IR.
    ///
    /// # Errors
    ///
    /// Returns an [`InferenceError`] if `ir` contains a construct inference does
    /// not yet support, or a function-like literal whose symbol is not present in
    /// `symbols` (an unbound or unlinked IR).
    pub fn infer<'symbols, S, E>(
        &mut self,
        symbols: &'symbols SymbolTable<'arena, A>,
        file: &File,
        ir: IR<'source, SymbolId, S, E>,
        extensions: Extensions<'arena, A>,
    ) -> InferenceResult<IR<'arena, SymbolId, Flow, Type<'arena>>> {
        InferenceFolder::new(self.source, self.arena, symbols, file, extensions).infer_ir(ir)
    }
}
