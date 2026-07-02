use mago_allocator::prelude::*;
use mago_flags::U8Flags;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::array::ArrayAtom;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::atom::payload::object::named::ObjectAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::well_known;

fn build_rich_type<'arena, S, A>(builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let label = builder.intern(b"label");
    let hello = builder.intern(b"hello");
    let literal = builder.string(StringAtom {
        literal: StringLiteral::Value(hello),
        casing: StringCasing::Unspecified,
        flags: U8Flags::empty(),
    });
    let literal_type = builder.union_of(&[literal]);
    let items =
        builder.known_items(&[KnownItem { key: ArrayKey::String(label), value: literal_type, optional: false }]);
    let shape = builder.array(ArrayAtom {
        key_param: None,
        value_param: None,
        known_items: Some(items),
        flags: U8Flags::empty(),
    });

    let collection_name = builder.intern_class_like_path(b"Collection");
    let arguments = builder.types(&[well_known::TYPE_INT, literal_type]);
    let collection =
        builder.object(ObjectAtom { name: collection_name, type_arguments: Some(arguments), flags: U8Flags::empty() });

    let negatable = builder.union_of(&[well_known::FALSE]);
    let negated = builder.negated(negatable);

    builder.union_of(&[well_known::NULL, shape, collection, negated])
}

#[test]
fn copy_into_round_trips_across_arenas() {
    let source_arena = LocalArena::new();
    let source_scratch = LocalArena::new();
    let mut source = TypeBuilder::new(&source_arena, &source_scratch);
    let original = build_rich_type(&mut source);

    let target = LocalArena::new();
    let copied = original.copy_into(&target);

    assert_eq!(copied, original);
    assert_eq!(copied.to_string(), original.to_string());
    assert_eq!(copied.kinds, original.kinds);
    assert!(!copied.ptr_eq(&original));
}

#[test]
fn copy_of_copy_remains_equal() {
    let source_arena = LocalArena::new();
    let source_scratch = LocalArena::new();
    let mut source = TypeBuilder::new(&source_arena, &source_scratch);
    let original = build_rich_type(&mut source);

    let first_arena = LocalArena::new();
    let second_arena = LocalArena::new();
    let first = original.copy_into(&first_arena);
    let second = first.copy_into(&second_arena);

    assert_eq!(second, original);
    assert_eq!(second.to_string(), original.to_string());
}

#[test]
fn import_after_copy_restores_consing() {
    let source_arena = LocalArena::new();
    let source_scratch = LocalArena::new();
    let mut source = TypeBuilder::new(&source_arena, &source_scratch);
    let original = build_rich_type(&mut source);

    let copy_arena = LocalArena::new();
    let copied = original.copy_into(&copy_arena);

    let imported = source.import(copied);
    assert!(imported.ptr_eq(&original));
}

#[test]
fn covariance_lets_static_values_mix_with_arena_values() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let name = builder.intern_class_like_path(b"Foo");
    let foo = builder.object(ObjectAtom { name, type_arguments: None, flags: U8Flags::empty() });
    let mixed_origin = builder.union_of(&[well_known::STRING, foo]);

    assert!(mixed_origin.contains_kind(mago_oracle::ty::AtomKind::String));
    assert!(mixed_origin.contains_kind(mago_oracle::ty::AtomKind::Object));
    assert_eq!(atom_count(well_known::TYPE_INT, mixed_origin), 3);
}

fn atom_count<'arena>(first: Type<'arena>, second: Type<'arena>) -> usize {
    first.atoms.len() + second.atoms.len()
}

#[test]
fn well_known_atoms_copy_to_any_arena() {
    let arena = LocalArena::new();

    for atom in well_known::ATOMS {
        let copied: Atom<'_> = atom.copy_into(&arena);
        assert_eq!(copied, *atom);
        assert_eq!(copied.to_string(), atom.to_string());
    }

    for ty in well_known::types() {
        let copied: Type<'_> = ty.copy_into(&arena);
        assert_eq!(copied, ty);
        assert_eq!(copied.to_string(), ty.to_string());
    }
}
