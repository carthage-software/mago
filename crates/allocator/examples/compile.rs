//! End-to-end sketch of the parallel compilation pipeline an arena has to support:
//!
//! 1. parse + lower + scan — one task per file, in parallel.
//! 2. link — merge every definition table into one symbol table, serially.
//! 3. infer + check — one task per file, in parallel, against the shared table.
//!
//! The whole thing runs on a single [`SharedArena`] shared across the rayon
//! workers; every pass is generic over `A: Arena` and never learns whether the
//! arena it was handed is local or shared.

#![allow(clippy::inline_trait_bounds)]
#![allow(clippy::missing_assert_message)]
#![allow(clippy::wildcard_imports)]

use std::marker::PhantomData;

use mago_allocator::prelude::*;
use rayon::prelude::*;

struct Ast<'arena> {
    _phantom: PhantomData<&'arena ()>,
}

struct Ir<'arena, T> {
    _phantom: PhantomData<&'arena T>,
}

struct DefinitionTable<'arena> {
    _phantom: PhantomData<&'arena ()>,
}

struct SymbolTable<'arena, A: Arena> {
    _symbols: HashMap<'arena, u32, DefinitionTable<'arena>, A>,
}

struct Type<'arena> {
    _phantom: PhantomData<&'arena ()>,
}

fn parse<'arena, A: Arena>(arena: &'arena A, _source: &str) -> &'arena Ast<'arena> {
    arena.alloc(Ast { _phantom: PhantomData })
}

fn lower<'arena, A: Arena>(arena: &'arena A, _ast: &Ast<'arena>) -> &'arena Ir<'arena, ()> {
    arena.alloc(Ir { _phantom: PhantomData })
}

fn scan<'arena, A: Arena>(_arena: &'arena A, _ir: &Ir<'arena, ()>) -> DefinitionTable<'arena> {
    DefinitionTable { _phantom: PhantomData }
}

fn link<'arena, A: Arena>(
    arena: &'arena A,
    definitions: Vec<'arena, DefinitionTable<'arena>, A>,
) -> SymbolTable<'arena, A> {
    let mut symbols: HashMap<'arena, u32, DefinitionTable<'arena>, A> = HashMap::new_in(arena);
    for (index, definition) in definitions.into_iter().enumerate() {
        symbols.insert(index as u32, definition);
    }

    SymbolTable { _symbols: symbols }
}

fn inference<'arena, A: Arena>(
    arena: &'arena A,
    _symbol_table: &SymbolTable<'arena, A>,
    _ir: &Ir<'arena, ()>,
) -> &'arena Ir<'arena, Type<'arena>> {
    arena.alloc(Ir { _phantom: PhantomData })
}

fn check<'arena, A: Arena>(_arena: &'arena A, _ir: &Ir<'arena, Type<'arena>>) -> bool {
    true
}

fn compile<'arena>(arena: &'arena SharedArena, sources: &[String]) -> bool {
    let ir_and_defs: Vec<'arena, _, SharedArena> = sources
        .par_iter()
        .map(|source| {
            let ast = parse(arena, source);
            let ir = lower(arena, ast);
            let definition_table = scan(arena, ir);

            (ir, definition_table)
        })
        .collect_in(arena);

    let mut irs = Vec::with_capacity_in(ir_and_defs.len(), arena);
    let mut defs = Vec::with_capacity_in(ir_and_defs.len(), arena);
    for (ir, definition_table) in ir_and_defs {
        irs.push(ir);
        defs.push(definition_table);
    }

    let symbol_table = link(arena, defs);

    let results = irs
        .as_slice()
        .par_iter()
        .map(|&ir| {
            let ir_with_types = inference(arena, &symbol_table, ir);

            check(arena, ir_with_types)
        })
        .collect_in(arena);

    results.into_iter().all(|result| result)
}

fn main() {
    let arena = SharedArena::new();
    let sources = [String::from("<?php echo 1;"), String::from("<?php echo 2;")];

    assert!(compile(&arena, &sources));
}
