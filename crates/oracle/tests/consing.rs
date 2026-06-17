use mago_allocator::LocalArena;
use mago_flags::U8Flags;
use mago_oracle::ty::Atom;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::object::named::ObjectAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::well_known;

#[test]
fn union_of_is_idempotent_by_pointer() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let name = builder.intern_class_like_path(b"Foo");
    let foo = builder.object(ObjectAtom { name, type_arguments: None, flags: U8Flags::empty() });

    let first = builder.union_of(&[well_known::NULL, foo]);
    let second = builder.union_of(&[well_known::NULL, foo]);

    assert_eq!(first, second);
    assert!(first.ptr_eq(&second));
}

#[test]
fn union_of_sorts_and_dedupes() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let unsorted = builder.union_of(&[well_known::STRING, well_known::INT, well_known::STRING]);
    let sorted = builder.union_of(&[well_known::INT, well_known::STRING]);

    assert_eq!(unsorted, sorted);
    assert!(unsorted.ptr_eq(&sorted));
    assert_eq!(unsorted.atoms, &[well_known::INT, well_known::STRING]);
}

#[test]
fn union_of_normalizes_well_known_singletons() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let int = builder.union_of(&[well_known::INT]);
    assert!(int.ptr_eq(&well_known::TYPE_INT));

    let null_or_string = builder.union_of(&[well_known::STRING, well_known::NULL]);
    assert!(null_or_string.ptr_eq(&well_known::TYPE_NULL_OR_STRING));
}

#[test]
fn union_of_empty_collapses_to_never() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let never = builder.union_of(&[]);

    assert!(never.is_never());
    assert!(never.ptr_eq(&well_known::TYPE_NEVER));
}

#[test]
fn payload_consing_is_idempotent_and_seeded_with_well_knowns() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let plain = StringAtom { literal: StringLiteral::None, casing: StringCasing::Unspecified, flags: U8Flags::empty() };
    let first = builder.string(plain);
    let second = builder.string(plain);

    assert_eq!(first, well_known::STRING);

    let (Atom::String(first_payload), Atom::String(second_payload)) = (first, second) else {
        panic!("both atoms must be strings");
    };
    assert!(std::ptr::eq(first_payload, second_payload));

    let singleton = builder.union_of(&[first]);
    assert!(singleton.ptr_eq(&well_known::TYPE_STRING));
}

#[test]
fn int_range_consing_is_idempotent() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let first = builder.int_range(Some(1), None);
    let second = builder.int_range(Some(1), None);

    assert_eq!(first, well_known::POSITIVE_INT);

    let (Atom::Int(IntAtom::Range(first_range)), Atom::Int(IntAtom::Range(second_range))) = (first, second) else {
        panic!("both atoms must be int ranges");
    };
    assert!(std::ptr::eq(first_range, second_range));
}

#[test]
fn names_are_interned_once() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let first = builder.intern(b"Vendor\\Collection");
    let second = builder.intern(b"Vendor\\Collection");

    assert_eq!(first, second);
    assert!(first.as_ptr() == second.as_ptr());
}

#[test]
fn intersected_with_empty_conjuncts_returns_head() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let name = builder.intern_class_like_path(b"Foo");
    let foo = builder.object(ObjectAtom { name, type_arguments: None, flags: U8Flags::empty() });
    let intersected = builder.intersected(foo, &[]);

    assert_eq!(intersected, foo);
}

#[test]
fn intersected_sorts_and_dedupes_conjuncts() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let foo_name = builder.intern_class_like_path(b"Foo");
    let bar_name = builder.intern_class_like_path(b"Bar");
    let qux_name = builder.intern_class_like_path(b"Qux");
    let foo = builder.object(ObjectAtom { name: foo_name, type_arguments: None, flags: U8Flags::empty() });
    let bar = builder.object(ObjectAtom { name: bar_name, type_arguments: None, flags: U8Flags::empty() });
    let qux = builder.object(ObjectAtom { name: qux_name, type_arguments: None, flags: U8Flags::empty() });

    let first = builder.intersected(foo, &[bar, qux, bar]);
    let second = builder.intersected(foo, &[qux, bar]);

    assert_eq!(first, second);
    assert_eq!(first.intersection_types(), second.intersection_types());
    assert!(first.has_intersection_types());
}

#[test]
fn import_within_one_builder_is_identity_by_pointer() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let name = builder.intern_class_like_path(b"Foo");
    let arguments = builder.types(&[well_known::TYPE_INT, well_known::TYPE_STRING]);
    let foo = builder.object(ObjectAtom { name, type_arguments: Some(arguments), flags: U8Flags::empty() });
    let ty = builder.union_of(&[well_known::NULL, foo]);

    let imported = builder.import(ty);

    assert!(imported.ptr_eq(&ty));
}

#[test]
fn import_across_arenas_preserves_structure() {
    let source_arena = LocalArena::new();
    let source_scratch = LocalArena::new();
    let mut source = TypeBuilder::new(&source_arena, &source_scratch);

    let name = source.intern_class_like_path(b"Collection");
    let value = source.intern(b"item");
    let literal = source.string(StringAtom {
        literal: StringLiteral::Value(value),
        casing: StringCasing::Unspecified,
        flags: U8Flags::empty(),
    });
    let item = source.union_of(&[literal]);
    let arguments = source.types(&[item, well_known::TYPE_INT]);
    let collection = source.object(ObjectAtom { name, type_arguments: Some(arguments), flags: U8Flags::empty() });
    let ty = source.union_of(&[well_known::NULL, collection]);

    let target_arena = LocalArena::new();
    let target_scratch = LocalArena::new();
    let mut target = TypeBuilder::new(&target_arena, &target_scratch);

    let imported = target.import(ty);

    assert_eq!(imported, ty);
    assert_eq!(imported.to_string(), ty.to_string());
    assert_eq!(imported.kinds, ty.kinds);
}

#[test]
fn import_twice_is_pointer_idempotent() {
    let source_arena = LocalArena::new();
    let source_scratch = LocalArena::new();
    let mut source = TypeBuilder::new(&source_arena, &source_scratch);

    let name = source.intern_class_like_path(b"Foo");
    let foo = source.object(ObjectAtom { name, type_arguments: None, flags: U8Flags::empty() });
    let ty = source.union_of(&[well_known::NULL, foo]);

    let target_arena = LocalArena::new();
    let target_scratch = LocalArena::new();
    let mut target = TypeBuilder::new(&target_arena, &target_scratch);

    let first = target.import(ty);
    let second = target.import(ty);

    assert!(first.ptr_eq(&second));
}
