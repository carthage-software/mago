use mago_allocator::LocalArena;
use mago_flags::U8Flags;
use mago_oracle::ty::Atom;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::array::ArrayAtom;
use mago_oracle::ty::atom::payload::array::ListAtom;
use mago_oracle::ty::atom::payload::array::ListFlag;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::atom::payload::callable::Parameter;
use mago_oracle::ty::atom::payload::callable::Signature;
use mago_oracle::ty::atom::payload::iterable::IterableAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::well_known;

#[test]
fn primitives_render_as_keywords() {
    assert_eq!(well_known::TYPE_INT.to_string(), "int");
    assert_eq!(well_known::TYPE_FLOAT.to_string(), "float");
    assert_eq!(well_known::TYPE_STRING.to_string(), "string");
    assert_eq!(well_known::TYPE_BOOL.to_string(), "bool");
    assert_eq!(well_known::TYPE_NULL.to_string(), "null");
    assert_eq!(well_known::TYPE_VOID.to_string(), "void");
    assert_eq!(well_known::TYPE_NEVER.to_string(), "never");
    assert_eq!(well_known::TYPE_MIXED.to_string(), "mixed");
    assert_eq!(well_known::TYPE_OBJECT.to_string(), "object");
    assert_eq!(well_known::TYPE_ARRAY_KEY.to_string(), "array-key");
    assert_eq!(well_known::TYPE_NUMERIC.to_string(), "numeric");
    assert_eq!(well_known::TYPE_SCALAR.to_string(), "scalar");
}

#[test]
fn mixed_refinements_render() {
    assert_eq!(well_known::MIXED.to_string(), "mixed");
    assert_eq!(well_known::NON_NULL_MIXED.to_string(), "nonnull");
    assert_eq!(well_known::TRUTHY_MIXED.to_string(), "truthy-mixed");
    assert_eq!(well_known::FALSY_MIXED.to_string(), "falsy-mixed");
    assert_eq!(well_known::ISSET_FROM_LOOP.to_string(), "mixed");
}

#[test]
fn int_literal_renders_as_value() {
    assert_eq!(Atom::Int(IntAtom::Literal(42)).to_string(), "int(42)");
    assert_eq!(Atom::Int(IntAtom::Literal(-1)).to_string(), "int(-1)");
}

#[test]
fn int_range_named_forms() {
    assert_eq!(well_known::POSITIVE_INT.to_string(), "positive-int");
    assert_eq!(well_known::NEGATIVE_INT.to_string(), "negative-int");
    assert_eq!(well_known::NON_NEGATIVE_INT.to_string(), "non-negative-int");
    assert_eq!(well_known::NON_POSITIVE_INT.to_string(), "non-positive-int");
    assert_eq!(well_known::LITERAL_INT.to_string(), "literal-int");
}

#[test]
fn int_range_with_bounds() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    assert_eq!(builder.int_range_atom(Some(0), Some(10)).to_string(), "int<0, 10>");
    assert_eq!(builder.int_range_atom(Some(5), None).to_string(), "int<5, max>");
    assert_eq!(builder.int_range_atom(None, Some(100)).to_string(), "int<min, 100>");
}

#[test]
fn string_literal_renders_quoted() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let value = builder.intern(b"hello");
    let literal = builder.string(StringAtom {
        literal: StringLiteral::Value(value),
        casing: StringCasing::Unspecified,
        flags: U8Flags::empty(),
    });

    assert_eq!(literal.to_string(), "string('hello')");
}

#[test]
fn refined_strings_render() {
    assert_eq!(well_known::NON_EMPTY_STRING.to_string(), "non-empty-string");
    assert_eq!(well_known::NUMERIC_STRING.to_string(), "numeric-string");
    assert_eq!(well_known::LOWERCASE_STRING.to_string(), "lowercase-string");
    assert_eq!(well_known::UPPERCASE_STRING.to_string(), "uppercase-string");
    assert_eq!(well_known::TRUTHY_STRING.to_string(), "truthy-string");
    assert_eq!(well_known::TRUTHY_NUMERIC_STRING.to_string(), "truthy-numeric-string");
    assert_eq!(well_known::CALLABLE_STRING.to_string(), "callable-string");
    assert_eq!(well_known::LITERAL_STRING.to_string(), "literal-string");
    assert_eq!(well_known::NON_EMPTY_LITERAL_STRING.to_string(), "non-empty-literal-string");
    assert_eq!(well_known::EMPTY_STRING.to_string(), "string('')");
}

#[test]
fn class_like_string_renders() {
    assert_eq!(well_known::CLASS_STRING.to_string(), "class-string");
    assert_eq!(well_known::INTERFACE_STRING.to_string(), "interface-string");
    assert_eq!(well_known::ENUM_STRING.to_string(), "enum-string");
    assert_eq!(well_known::TRAIT_STRING.to_string(), "trait-string");
}

#[test]
fn named_object_renders() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let name = builder.intern_class_like_path(b"Foo");
    let foo = builder.object(ObjectAtom { name, type_arguments: None, flags: U8Flags::empty() });

    assert_eq!(foo.to_string(), "Foo");
}

#[test]
fn generic_object_renders_with_args() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let name = builder.intern_class_like_path(b"Box");
    let arguments = builder.types(&[well_known::TYPE_INT]);
    let boxed = builder.object(ObjectAtom { name, type_arguments: Some(arguments), flags: U8Flags::empty() });

    assert_eq!(boxed.to_string(), "Box<int>");
}

#[test]
fn intersected_object_renders_with_amp() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let foo_name = builder.intern_class_like_path(b"Foo");
    let bar_name = builder.intern_class_like_path(b"Bar");
    let foo = builder.object(ObjectAtom { name: foo_name, type_arguments: None, flags: U8Flags::empty() });
    let bar = builder.object(ObjectAtom { name: bar_name, type_arguments: None, flags: U8Flags::empty() });
    let intersected = builder.intersected(foo, &[bar]);

    assert_eq!(intersected.to_string(), "Foo&Bar");
}

#[test]
fn union_renders_with_pipe() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let int_or_string = builder.union_of(&[well_known::INT, well_known::STRING]);

    assert_eq!(int_or_string.to_string(), "int|string");
}

#[test]
fn nullable_renders() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let nullable_int = builder.union_of(&[well_known::NULL, well_known::INT]);
    let rendered = nullable_int.to_string();

    assert!(rendered.contains("null") && rendered.contains("int"));
}

#[test]
fn list_renders() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let list = builder.list(ListAtom {
        element_type: well_known::TYPE_INT,
        known_elements: None,
        known_count: None,
        flags: U8Flags::empty(),
    });

    assert_eq!(list.to_string(), "list<int>");
}

#[test]
fn non_empty_list_renders() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let list = builder.list(ListAtom {
        element_type: well_known::TYPE_INT,
        known_elements: None,
        known_count: None,
        flags: U8Flags::empty().with(ListFlag::NonEmpty),
    });

    assert_eq!(list.to_string(), "non-empty-list<int>");
}

#[test]
fn keyed_array_unsealed_renders() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let array = builder.array(ArrayAtom {
        key_param: Some(well_known::TYPE_STRING),
        value_param: Some(well_known::TYPE_INT),
        known_items: None,
        flags: U8Flags::empty(),
    });

    assert_eq!(array.to_string(), "array<string, int>");
}

#[test]
fn iterable_renders() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let iterable =
        builder.iterable(IterableAtom { key_type: well_known::TYPE_STRING, value_type: well_known::TYPE_INT });

    assert_eq!(iterable.to_string(), "iterable<string, int>");
    assert_eq!(well_known::ITERABLE_MIXED_MIXED.to_string(), "iterable<mixed, mixed>");
}

#[test]
fn callable_signature_renders() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let parameter_name = builder.intern(b"value");
    let parameters = builder.parameters(&[Parameter {
        name: parameter_name,
        r#type: well_known::TYPE_INT,
        flags: U8Flags::empty(),
    }]);
    let signature = builder.signature(Signature {
        parameters: Some(parameters),
        return_type: well_known::TYPE_STRING,
        throws: None,
        flags: U8Flags::empty(),
    });

    assert_eq!(Atom::Callable(CallableAtom::Signature(signature)).to_string(), "(callable(int): string)");
    assert_eq!(Atom::Callable(CallableAtom::Closure(signature)).to_string(), "(closure(int): string)");
    assert_eq!(well_known::CALLABLE.to_string(), "callable");
}

#[test]
fn resource_variants_render() {
    assert_eq!(well_known::RESOURCE.to_string(), "resource");
    assert_eq!(well_known::OPEN_RESOURCE.to_string(), "open-resource");
    assert_eq!(well_known::CLOSED_RESOURCE.to_string(), "closed-resource");
}

#[test]
fn empty_array_renders() {
    assert_eq!(well_known::EMPTY_ARRAY.to_string(), "array{}");
    assert_eq!(well_known::ARRAY_KEY_MIXED.to_string(), "array<array-key, mixed>");
}

#[test]
fn intersection_queries_dispatch_on_atoms() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    assert!(well_known::INT.can_be_intersected());
    assert!(!well_known::INT.has_intersection_types());
    assert!(well_known::INT.intersection_types().is_empty());

    let foo_name = builder.intern_class_like_path(b"Foo");
    let bar_name = builder.intern_class_like_path(b"Bar");
    let foo = builder.object(ObjectAtom { name: foo_name, type_arguments: None, flags: U8Flags::empty() });
    let bar = builder.object(ObjectAtom { name: bar_name, type_arguments: None, flags: U8Flags::empty() });

    assert!(foo.can_be_intersected());
    assert!(!foo.has_intersection_types());

    let intersected = builder.intersected(foo, &[bar]);
    assert!(intersected.has_intersection_types());
    assert_eq!(intersected.intersection_types(), &[bar]);
    assert!(!intersected.can_be_intersected());
}
