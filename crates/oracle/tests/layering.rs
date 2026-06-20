mod common;

use common::symbol_table;

use mago_allocator::LocalArena;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::refines;
use mago_oracle::ty::well_known;

/// A long-lived [`SymbolTable`](mago_oracle::symbol::SymbolTable) (in its own
/// arena) answers queries for shorter file lifetimes covariantly: a
/// `&SymbolTable<'shared>` coerces to `&SymbolTable<'file>` whenever `'shared`
/// outlives `'file`, and the `Type<'shared>` it hands back coerces into the file
/// arena's types. No trait, no copying - plain subtyping.
#[test]
fn symbol_table_types_serve_shorter_file_lifetimes_covariantly() {
    let symbols_arena = LocalArena::new();
    let symbols = symbol_table(
        &symbols_arena,
        "<?php
class Collection {}
class ArrayCollection extends Collection {}
/** @var Collection|null */
const make_collection = null;",
    );

    let Some(declared) = symbols.global_constant_type(SymbolId::constant(b"make_collection")) else {
        panic!("the symbol table must know make_collection");
    };

    for _file in 0..3 {
        let file_arena = LocalArena::new();
        let file_scratch = LocalArena::new();
        let mut file_builder = TypeBuilder::new(&file_arena, &file_scratch);

        let array_collection = file_builder.object_named(b"ArrayCollection");
        let inferred = file_builder.union_of(&[array_collection]);

        let mut report = LatticeReport::new();
        assert!(refines(inferred, declared, &symbols, LatticeOptions::default(), &mut report, &mut file_builder));

        let mixed_union = file_builder.union_of(&[well_known::NULL, array_collection]);
        let mut report = LatticeReport::new();
        assert!(refines(mixed_union, declared, &symbols, LatticeOptions::default(), &mut report, &mut file_builder));

        let stranger = file_builder.object_named(b"Stranger");
        let stranger_type = file_builder.union_of(&[stranger]);
        let mut report = LatticeReport::new();
        assert!(!refines(stranger_type, declared, &symbols, LatticeOptions::default(), &mut report, &mut file_builder));
    }
}

#[test]
fn symbol_table_atoms_embed_into_file_types_without_copying() {
    let symbols_arena = LocalArena::new();
    let symbols_scratch = LocalArena::new();
    let mut symbols_builder = TypeBuilder::new(&symbols_arena, &symbols_scratch);

    let collection = symbols_builder.object_named(b"Collection");
    let symbols_type = symbols_builder.union_of(&[collection]);

    let file_arena = LocalArena::new();
    let file_scratch = LocalArena::new();
    let mut file_builder = TypeBuilder::new(&file_arena, &file_scratch);

    let mut atoms = symbols_type.atoms.to_vec();
    atoms.push(well_known::NULL);
    let file_type = file_builder.union_of(&atoms);

    assert_eq!(file_type.to_string(), "Collection|null");
    assert!(file_type.atoms.contains(&collection));
}

#[test]
fn imported_symbol_table_types_are_consed_in_the_file_arena() {
    let symbols_arena = LocalArena::new();
    let symbols_scratch = LocalArena::new();
    let mut symbols_builder = TypeBuilder::new(&symbols_arena, &symbols_scratch);

    let collection = symbols_builder.object_named(b"Collection");
    let symbols_type = symbols_builder.union_of(&[well_known::NULL, collection]);

    let file_arena = LocalArena::new();
    let file_scratch = LocalArena::new();
    let mut file_builder = TypeBuilder::new(&file_arena, &file_scratch);

    let imported = file_builder.import(symbols_type);
    let again = file_builder.import(symbols_type);

    assert_eq!(imported, symbols_type);
    assert!(imported.ptr_eq(&again));
}
