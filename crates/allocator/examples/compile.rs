#![allow(clippy::inline_trait_bounds)]
#![allow(clippy::missing_assert_message)]
#![allow(clippy::wildcard_imports)]

use std::marker::PhantomData;

use rayon::prelude::*;

use mago_allocator::prelude::*;

struct Ast<'arena> {
    _phantom: PhantomData<&'arena ()>,
}

struct Ir<'arena, T> {
    _phantom: PhantomData<&'arena T>,
}

struct DefinitionTable<'arena> {
    _phantom: PhantomData<&'arena ()>,
}

impl CopyInto for DefinitionTable<'_> {
    type Output<'arena> = DefinitionTable<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        DefinitionTable { _phantom: PhantomData }
    }
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

fn lower<'ast, 'arena, A: Arena>(arena: &'arena A, _ast: &'ast Ast<'ast>) -> &'arena Ir<'arena, ()> {
    arena.alloc(Ir { _phantom: PhantomData })
}

fn scan<'ir, 'arena, A: Arena>(_arena: &'arena A, _ir: &'ir Ir<'ir, ()>) -> DefinitionTable<'arena> {
    DefinitionTable { _phantom: PhantomData }
}

fn link<'defs, 'arena, A: Arena>(
    arena: &'arena A,
    definitions: &'defs [DefinitionTable<'defs>],
) -> SymbolTable<'arena, A> {
    let mut symbols: HashMap<'arena, u32, DefinitionTable<'arena>, A> = HashMap::new_in(arena);
    for (index, definition) in definitions.iter().enumerate() {
        symbols.insert(index as u32, definition.copy_into(arena));
    }

    SymbolTable { _symbols: symbols }
}

fn inference<'arena, A: Arena, S: Arena>(
    arena: &'arena A,
    scratch: &S,
    _symbol_table: &SymbolTable<'arena, A>,
    _ir: &Ir<'_, ()>,
) -> &'arena Ir<'arena, Type<'arena>> {
    let _ = scratch;
    arena.alloc(Ir { _phantom: PhantomData })
}

fn check<'arena>(_ir: &Ir<'arena, Type<'arena>>) -> bool {
    true
}

fn compile(arena: &SharedArena, sources: &[&str]) -> bool {
    let ir_arena = SharedArena::with_chunk_size(2 * 1024 * 1024);
    let definitions_arena = SharedArena::new();

    let parse_and_scan_file = |scratch: &mut ScopedArena<'_>, source: &&str| {
        let ast = parse(&*scratch, source);
        let ir = lower(&ir_arena, ast);
        let definition_table = scan(&definitions_arena, ir);
        scratch.reset();
        (ir, definition_table)
    };

    let ir_and_definitions: Vec<'_, _, SharedArena> =
        sources.par_iter().map_init(|| ir_arena.scoped(), parse_and_scan_file).collect_in(&ir_arena);

    let file_count = ir_and_definitions.len();
    let mut untyped_irs: Vec<'_, &Ir<'_, ()>, SharedArena> = Vec::with_capacity_in(file_count, &ir_arena);
    let mut definition_tables: Vec<'_, DefinitionTable<'_>, SharedArena> = Vec::with_capacity_in(file_count, &ir_arena);
    for (ir, definition_table) in ir_and_definitions {
        untyped_irs.push(ir);
        definition_tables.push(definition_table);
    }

    let symbol_table = link(arena, definition_tables.as_slice());
    drop(definition_tables);
    drop(definitions_arena);

    let infer_and_check_file = |scratch: &mut ScopedArena<'_>, ir: &&Ir<'_, ()>| {
        let typed_ir = inference(arena, &*scratch, &symbol_table, ir);
        scratch.reset();
        check(typed_ir)
    };

    let all_files_passed = untyped_irs
        .as_slice()
        .par_iter()
        .map_init(|| arena.scoped(), infer_and_check_file)
        .all(|file_passed| file_passed);

    drop(untyped_irs);
    drop(ir_arena);

    all_files_passed
}

fn main() {
    let arena = SharedArena::new();
    assert!(compile(&arena, &["<?php echo 1;", "<?php echo 2;"]));
}
