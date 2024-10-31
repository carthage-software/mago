use fennec_ast::Program;
use fennec_interner::ThreadedInterner;
use fennec_walker::MutWalker;

use crate::context::Context;
use crate::table::SymbolTable;
use crate::walker::SymbolWalker;

mod context;
mod walker;

pub mod symbol;
pub mod table;

/// Construct a symbol table from a program.
///
/// # Parameters
///
/// - `interner`: The interner to use for string interning.
/// - `program`: The program to construct the symbol table from.
/// - `names`: The resolved names for the program.
///
/// # Returns
///
/// A symbol table containing all the symbols in the program.
pub fn get_symbols(interner: &ThreadedInterner, program: &Program) -> SymbolTable {
    let mut walker = SymbolWalker::new();

    let mut context = Context::new(&interner);

    walker.walk_program(program, &mut context);

    walker.symbols
}
