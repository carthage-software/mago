use mago_allocator::LocalArena;
use mago_inference::reconciler::reconcile;
use mago_oracle::assertion::Assertion;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::well_known::BOOL;
use mago_oracle::ty::well_known::NULL;
use mago_oracle::ty::well_known::STRING;

#[test]
fn identical_meets_to_the_asserted_atom() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let symbols = SymbolTable::new_in(&arena);
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let nullable = builder.union_of(&[NULL, STRING]);
    let narrowed = reconcile(&mut builder, &symbols, Assertion::IsIdentical(NULL), nullable);

    assert_eq!(format!("{narrowed}"), "null");
}

#[test]
fn not_identical_subtracts_the_atom() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let symbols = SymbolTable::new_in(&arena);
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let nullable = builder.union_of(&[NULL, STRING]);
    let narrowed = reconcile(&mut builder, &symbols, Assertion::IsNotIdentical(NULL), nullable);

    assert_eq!(format!("{narrowed}"), "string");
}

#[test]
fn truthy_drops_falsy_atoms_and_refines_bool() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let symbols = SymbolTable::new_in(&arena);
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let nullable = builder.union_of(&[NULL, STRING]);
    assert_eq!(
        format!("{}", reconcile(&mut builder, &symbols, Assertion::Truthy, nullable)),
        "string&!string('')&!string('0')",
    );

    let boolean = builder.union_of(&[BOOL]);
    assert_eq!(format!("{}", reconcile(&mut builder, &symbols, Assertion::Truthy, boolean)), "true");
}

#[test]
fn falsy_keeps_falsy_atoms_and_refines_bool() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let symbols = SymbolTable::new_in(&arena);
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let boolean = builder.union_of(&[BOOL]);
    assert_eq!(format!("{}", reconcile(&mut builder, &symbols, Assertion::Falsy, boolean)), "false");
}

#[test]
fn isset_removes_null_and_not_isset_keeps_only_null() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let symbols = SymbolTable::new_in(&arena);
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let nullable = builder.union_of(&[NULL, STRING]);
    assert_eq!(format!("{}", reconcile(&mut builder, &symbols, Assertion::IsIsset, nullable)), "string");
    assert_eq!(format!("{}", reconcile(&mut builder, &symbols, Assertion::IsNotIsset, nullable)), "null");
}
