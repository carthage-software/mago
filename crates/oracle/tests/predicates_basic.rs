mod common;

use common::*;

use mago_oracle::ty::atom::payload::alias::AliasAtom;
use mago_oracle::ty::predicates;
use mago_oracle::ty::well_known;

#[test]
fn is_never_true_only_for_type_never() {
    assert!(predicates::is_never(well_known::TYPE_NEVER));
    assert!(!predicates::is_never(well_known::TYPE_INT));
    assert!(!predicates::is_never(well_known::TYPE_MIXED));
}

#[test]
fn is_mixed_true_only_for_vanilla_mixed() {
    fixture(|f| {
        assert!(predicates::is_mixed(well_known::TYPE_MIXED));
        assert!(!predicates::is_mixed(well_known::TYPE_INT));
        let truthy_mixed = f.mixed_truthy();
        let truthy_mixed_type = f.u(truthy_mixed);
        assert!(!predicates::is_mixed(truthy_mixed_type));
    });
}

#[test]
fn is_singleton_and_is_union() {
    fixture(|f| {
        assert!(predicates::is_singleton(well_known::TYPE_INT));
        assert!(!predicates::is_union(well_known::TYPE_INT));
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        assert!(!predicates::is_singleton(int_or_string));
        assert!(predicates::is_union(int_or_string));
    });
}

#[test]
fn is_int_true_for_int_family() {
    fixture(|f| {
        assert!(predicates::is_int(well_known::TYPE_INT));
        let literal = f.ui(42);
        assert!(predicates::is_int(literal));
        let range = f.t_int_range(0, 10);
        let range_type = f.u(range);
        assert!(predicates::is_int(range_type));
        let positive = f.t_positive_int();
        let positive_type = f.u(positive);
        assert!(predicates::is_int(positive_type));
    });
}

#[test]
fn is_int_false_for_unions_with_other_kinds() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        assert!(!predicates::is_int(int_or_string));
        assert!(!predicates::is_int(well_known::TYPE_STRING));
        assert!(!predicates::is_int(well_known::TYPE_NEVER));
    });
}

#[test]
fn is_int_true_for_union_of_int_variants() {
    fixture(|f| {
        let zero = f.t_lit_int(0);
        let range = f.t_int_range(1, 10);
        let union = f.u_many(vec![zero, range]);
        assert!(predicates::is_int(union));
    });
}

#[test]
fn is_string_distinguishes_class_like_string() {
    fixture(|f| {
        assert!(predicates::is_string(well_known::TYPE_STRING));
        let non_empty = f.t_non_empty_string();
        let non_empty_type = f.u(non_empty);
        assert!(predicates::is_string(non_empty_type));
        let class_string = f.t_class_string();
        let class_string_type = f.u(class_string);
        assert!(!predicates::is_string(class_string_type));
    });
}

#[test]
fn is_bool_includes_true_false() {
    fixture(|f| {
        assert!(predicates::is_bool(well_known::TYPE_BOOL));
        assert!(predicates::is_bool(well_known::TYPE_TRUE));
        assert!(predicates::is_bool(well_known::TYPE_FALSE));
        let true_atom = f.t_true();
        let false_atom = f.t_false();
        let union = f.u_many(vec![true_atom, false_atom]);
        assert!(predicates::is_bool(union));
    });
}

#[test]
fn is_object_covers_object_family() {
    fixture(|f| {
        assert!(predicates::is_object(well_known::TYPE_OBJECT));
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        assert!(predicates::is_object(foo_type));
        let enum_atom = f.t_enum("E");
        let enum_type = f.u(enum_atom);
        assert!(predicates::is_object(enum_type));
    });
}

#[test]
fn is_array_covers_list_and_keyed() {
    fixture(|f| {
        let empty_array = f.t_empty_array();
        let empty_array_type = f.u(empty_array);
        assert!(predicates::is_array(empty_array_type));
        let list = f.t_list(well_known::TYPE_INT, false);
        let list_type = f.u(list);
        assert!(predicates::is_array(list_type));
        assert!(predicates::is_list(list_type));
        assert!(!predicates::is_list(empty_array_type));
    });
}

#[test]
fn is_callable_strict() {
    fixture(|f| {
        let callable = f.t_callable_mixed();
        let callable_type = f.u(callable);
        assert!(predicates::is_callable(callable_type));
        assert!(!predicates::is_callable(well_known::TYPE_STRING));
    });
}

#[test]
fn is_scalar_dominator_and_members() {
    fixture(|f| {
        assert!(predicates::is_scalar(well_known::TYPE_SCALAR));
        assert!(predicates::is_scalar(well_known::TYPE_INT));
        assert!(predicates::is_scalar(well_known::TYPE_STRING));
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        assert!(!predicates::is_scalar(foo_type));
    });
}

#[test]
fn is_numeric_includes_int_and_float() {
    assert!(predicates::is_numeric(well_known::TYPE_INT));
    assert!(predicates::is_numeric(well_known::TYPE_FLOAT));
    assert!(predicates::is_numeric(well_known::TYPE_NUMERIC));
    assert!(!predicates::is_numeric(well_known::TYPE_STRING));
}

#[test]
fn contains_string_in_union() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let union = f.u_many(vec![int, string]);
        assert!(predicates::contains_string(union));
        assert!(predicates::contains_int(union));
        assert!(!predicates::contains_float(union));
    });
}

#[test]
fn contains_null_in_nullable() {
    fixture(|f| {
        let null = f.null();
        let int = f.t_int();
        let union = f.u_many(vec![null, int]);
        assert!(predicates::contains_null(union));
    });
}

#[test]
fn contains_object_in_union() {
    fixture(|f| {
        let null = f.null();
        let foo = f.t_named("Foo");
        let union = f.u_many(vec![null, foo]);
        assert!(predicates::contains_object(union));
    });
}

#[test]
fn contains_mixed_top_level_only() {
    fixture(|f| {
        assert!(predicates::contains_mixed(well_known::TYPE_MIXED));
        let list = f.t_list(well_known::TYPE_MIXED, false);
        let nested = f.u(list);
        assert!(!predicates::contains_mixed(nested));
        assert!(predicates::contains_mixed_anywhere(nested));
    });
}

#[test]
fn true_is_truthy_false_is_falsy() {
    assert!(predicates::is_truthy(well_known::TYPE_TRUE));
    assert!(!predicates::is_truthy(well_known::TYPE_FALSE));
    assert!(predicates::is_falsy(well_known::TYPE_FALSE));
    assert!(!predicates::is_falsy(well_known::TYPE_TRUE));
}

#[test]
fn null_void_falsy() {
    assert!(predicates::is_falsy(well_known::TYPE_NULL));
    assert!(predicates::is_falsy(well_known::TYPE_VOID));
}

#[test]
fn object_always_truthy() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        assert!(predicates::is_truthy(foo_type));
        assert!(!predicates::is_falsy(foo_type));
    });
}

#[test]
fn callable_always_truthy() {
    fixture(|f| {
        let callable = f.t_callable_mixed();
        let callable_type = f.u(callable);
        assert!(predicates::is_truthy(callable_type));
    });
}

#[test]
fn class_like_string_truthy() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let class_string_type = f.u(class_string);
        assert!(predicates::is_truthy(class_string_type));
    });
}

#[test]
fn lit_int_zero_falsy_nonzero_truthy() {
    fixture(|f| {
        let zero = f.ui(0);
        assert!(predicates::is_falsy(zero));
        let forty_two = f.ui(42);
        assert!(predicates::is_truthy(forty_two));
        let minus_one = f.ui(-1);
        assert!(predicates::is_truthy(minus_one));
    });
}

#[test]
fn int_range_truthy_when_excludes_zero() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let positive_type = f.u(positive);
        assert!(predicates::is_truthy(positive_type));
        let negative = f.t_negative_int();
        let negative_type = f.u(negative);
        assert!(predicates::is_truthy(negative_type));
        let range = f.t_int_range(0, 10);
        let range_type = f.u(range);
        assert!(!predicates::is_truthy(range_type));
        assert!(!predicates::is_truthy(well_known::TYPE_INT));
    });
}

#[test]
fn empty_string_falsy_truthy_string_truthy() {
    fixture(|f| {
        let empty = f.us("");
        assert!(predicates::is_falsy(empty));
        let truthy = f.t_truthy_string();
        let truthy_type = f.u(truthy);
        assert!(predicates::is_truthy(truthy_type));
        let word = f.us("foo");
        assert!(predicates::is_truthy(word));
        let zero = f.us("0");
        assert!(
            !predicates::is_falsy(zero),
            "only known-empty literals are guaranteed falsy; \"0\" must go through widen::literals first"
        );
        assert!(!predicates::is_truthy(zero));
    });
}

#[test]
fn empty_array_falsy() {
    fixture(|f| {
        let empty_array = f.t_empty_array();
        let empty_array_type = f.u(empty_array);
        assert!(predicates::is_falsy(empty_array_type));
    });
}

#[test]
fn non_empty_list_truthy() {
    fixture(|f| {
        let list = f.t_list(well_known::TYPE_INT, true);
        let list_type = f.u(list);
        assert!(predicates::is_truthy(list_type));
    });
}

#[test]
fn resources_truthy_unless_explicitly_closed() {
    fixture(|f| {
        let open = f.t_open_resource();
        let open_type = f.u(open);
        assert!(predicates::is_truthy(open_type));
        let closed = f.t_closed_resource();
        let closed_type = f.u(closed);
        assert!(predicates::is_falsy(closed_type));
        let resource = f.t_resource();
        let resource_type = f.u(resource);
        assert!(
            predicates::is_truthy(resource_type),
            "unknown-state resources are truthy; only explicitly-closed resources are falsy"
        );
        assert!(!predicates::is_falsy(resource_type));
    });
}

#[test]
fn truthy_mixed_truthy() {
    fixture(|f| {
        let truthy_mixed = f.mixed_truthy();
        let truthy_mixed_type = f.u(truthy_mixed);
        assert!(predicates::is_truthy(truthy_mixed_type));
        let falsy_mixed = f.mixed_falsy();
        let falsy_mixed_type = f.u(falsy_mixed);
        assert!(predicates::is_falsy(falsy_mixed_type));
    });
}

#[test]
fn could_be_truthy_for_general_int() {
    assert!(predicates::could_be_truthy(well_known::TYPE_INT));
    assert!(predicates::could_be_falsy(well_known::TYPE_INT));
}

#[test]
fn could_be_truthy_excludes_never_and_void() {
    assert!(!predicates::could_be_truthy(well_known::TYPE_NEVER));
    assert!(!predicates::could_be_truthy(well_known::TYPE_VOID));
}

#[test]
fn could_be_falsy_excludes_never_but_includes_void() {
    assert!(!predicates::could_be_falsy(well_known::TYPE_NEVER));
    assert!(predicates::could_be_falsy(well_known::TYPE_VOID));
}

#[test]
fn nullable_int_could_be_both() {
    fixture(|f| {
        let null = f.null();
        let int = f.t_int();
        let nullable = f.u_many(vec![null, int]);
        assert!(predicates::could_be_truthy(nullable));
        assert!(predicates::could_be_falsy(nullable));
    });
}

#[test]
fn is_literal_true_for_known_values() {
    fixture(|f| {
        let int_literal = f.ui(42);
        assert!(predicates::is_literal(int_literal));
        let string_literal = f.us("foo");
        assert!(predicates::is_literal(string_literal));
        let float_literal = f.t_lit_float(1.5);
        let float_literal_type = f.u(float_literal);
        assert!(predicates::is_literal(float_literal_type));
        assert!(predicates::is_literal(well_known::TYPE_TRUE));
        assert!(predicates::is_literal(well_known::TYPE_FALSE));
        assert!(predicates::is_literal(well_known::TYPE_NULL));
        assert!(predicates::is_literal(well_known::TYPE_VOID));
    });
}

#[test]
fn is_literal_false_for_general_forms() {
    fixture(|f| {
        assert!(!predicates::is_literal(well_known::TYPE_INT));
        assert!(!predicates::is_literal(well_known::TYPE_STRING));
        let range = f.t_int_range(0, 10);
        let range_type = f.u(range);
        assert!(!predicates::is_literal(range_type));
        let non_empty = f.t_non_empty_string();
        let non_empty_type = f.u(non_empty);
        assert!(!predicates::is_literal(non_empty_type));
    });
}

#[test]
fn is_constant_foldable_requires_singleton() {
    fixture(|f| {
        let literal = f.ui(42);
        assert!(predicates::is_constant_foldable(literal));
        let one = f.t_lit_int(1);
        let two = f.t_lit_int(2);
        let union_literals = f.u_many(vec![one, two]);
        assert!(predicates::is_literal(union_literals));
        assert!(!predicates::is_constant_foldable(union_literals));
    });
}

#[test]
fn contains_template_anywhere_finds_nested() {
    fixture(|f| {
        let parameter = f.t_template_of("F", "T", well_known::TYPE_MIXED);
        let parameter_type = f.u(parameter);
        let list = f.t_list(parameter_type, false);
        let nested = f.u(list);
        assert!(predicates::contains_template_anywhere(nested));
        assert!(!predicates::contains_template_anywhere(well_known::TYPE_INT));
    });
}

#[test]
fn contains_unresolved_anywhere_finds_alias_in_list() {
    fixture(|f| {
        let class_name = f.name("Foo");
        let alias_name = f.builder.intern(b"Id");
        let alias = f.builder.alias(AliasAtom { class_name, alias_name });
        let alias_type = f.u(alias);
        let list = f.t_list(alias_type, false);
        let nested = f.u(list);
        assert!(predicates::contains_unresolved_anywhere(nested));
        assert!(!predicates::is_fully_resolved(nested));
        assert!(predicates::is_fully_resolved(well_known::TYPE_INT));
    });
}

#[test]
fn contains_mixed_anywhere_walks_into_object_args() {
    fixture(|f| {
        let boxed = f.t_generic_named("Box", vec![well_known::TYPE_MIXED]);
        let nested = f.u(boxed);
        assert!(predicates::contains_mixed_anywhere(nested));
    });
}
