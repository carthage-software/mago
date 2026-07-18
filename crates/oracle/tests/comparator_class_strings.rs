mod common;

use common::*;

#[test]
fn class_string_reflexive() {
    fixture(|f| {
        let class_string = f.t_class_string();
        assert_atomic_subtype(f, class_string, class_string);
    });
}

#[test]
fn interface_string_reflexive() {
    fixture(|f| {
        let interface_string = f.t_interface_string();
        assert_atomic_subtype(f, interface_string, interface_string);
    });
}

#[test]
fn enum_string_reflexive() {
    fixture(|f| {
        let enum_string = f.t_enum_string();
        assert_atomic_subtype(f, enum_string, enum_string);
    });
}

#[test]
fn trait_string_reflexive() {
    fixture(|f| {
        let trait_string = f.t_trait_string();
        assert_atomic_subtype(f, trait_string, trait_string);
    });
}

#[test]
fn lit_class_string_reflexive() {
    fixture(|f| {
        for name in ["Foo", "App\\Bar", "Vendor\\Pkg\\X"] {
            let literal = f.t_lit_class_string(name);
            assert_atomic_subtype(f, literal, literal);
        }
    });
}

#[test]
fn lit_class_string_in_class_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        for name in ["Foo", "Bar", "App\\Service"] {
            let literal = f.t_lit_class_string(name);
            assert_atomic_subtype(f, literal, class_string);
        }
    });
}

#[test]
fn class_string_not_in_lit_class_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let literal = f.t_lit_class_string("Foo");
        assert_atomic_not_subtype(f, class_string, literal);
    });
}

#[test]
fn class_string_in_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let string = f.t_string();
        assert_atomic_subtype(f, class_string, string);
    });
}

#[test]
fn interface_string_in_string() {
    fixture(|f| {
        let interface_string = f.t_interface_string();
        let string = f.t_string();
        assert_atomic_subtype(f, interface_string, string);
    });
}

#[test]
fn enum_string_in_string() {
    fixture(|f| {
        let enum_string = f.t_enum_string();
        let string = f.t_string();
        assert_atomic_subtype(f, enum_string, string);
    });
}

#[test]
fn trait_string_in_string() {
    fixture(|f| {
        let trait_string = f.t_trait_string();
        let string = f.t_string();
        assert_atomic_subtype(f, trait_string, string);
    });
}

#[test]
fn class_string_in_array_key() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let array_key = f.t_array_key();
        assert_atomic_subtype(f, class_string, array_key);
    });
}

#[test]
fn class_string_in_scalar() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, class_string, scalar);
    });
}

#[test]
fn class_string_not_in_int() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let int = f.t_int();
        assert_atomic_not_subtype(f, class_string, int);
    });
}

#[test]
fn class_string_not_in_numeric() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let numeric = f.t_numeric();
        assert_atomic_not_subtype(f, class_string, numeric);
    });
}

#[test]
fn lit_class_string_in_string() {
    fixture(|f| {
        let literal = f.t_lit_class_string("Foo");
        let string = f.t_string();
        assert_atomic_subtype(f, literal, string);
    });
}

#[test]
fn lit_class_string_in_array_key() {
    fixture(|f| {
        let literal = f.t_lit_class_string("Foo");
        let array_key = f.t_array_key();
        assert_atomic_subtype(f, literal, array_key);
    });
}

#[test]
fn distinct_lit_class_strings_disjoint() {
    fixture(|f| {
        let foo = f.t_lit_class_string("Foo");
        let bar = f.t_lit_class_string("Bar");
        assert_atomic_not_subtype(f, foo, bar);
    });
}

#[test]
fn many_lit_class_strings_in_class_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        for index in 0..30 {
            let name = format!("Class_{index}");
            let literal = f.t_lit_class_string(&name);
            assert_atomic_subtype(f, literal, class_string);
        }
    });
}

#[test]
fn lit_class_string_in_class_string_of_self() {
    fixture(|f| {
        let named = f.t_named("Foo");
        let named_type = f.u(named);
        let foo = f.t_class_string_of(named_type);
        let literal = f.t_lit_class_string("Foo");
        assert_atomic_subtype(f, literal, foo);
    });
}

#[test]
fn lit_class_string_in_class_string_of_ancestor() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo {} class Bar extends Foo {}");
        let named = f.t_named("Foo");
        let named_type = f.u(named);
        let foo = f.t_class_string_of(named_type);
        let literal = f.t_lit_class_string("Bar");
        assert!(atomic_is_contained(f, literal, foo, &symbols));
    });
}

#[test]
fn lit_class_string_not_in_class_string_of_unrelated() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let named = f.t_named("Foo");
        let named_type = f.u(named);
        let foo = f.t_class_string_of(named_type);
        let literal = f.t_lit_class_string("Bar");
        assert!(!atomic_is_contained(f, literal, foo, &symbols));
    });
}

#[test]
fn class_string_of_child_in_class_string_of_parent() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo {} class Bar extends Foo {}");
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let parent = f.t_class_string_of(foo_type);
        let bar = f.t_named("Bar");
        let bar_type = f.u(bar);
        let child = f.t_class_string_of(bar_type);
        assert!(atomic_is_contained(f, child, parent, &symbols));
    });
}

#[test]
fn class_string_of_parent_not_in_class_string_of_child() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo {} class Bar extends Foo {}");
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let parent = f.t_class_string_of(foo_type);
        let bar = f.t_named("Bar");
        let bar_type = f.u(bar);
        let child = f.t_class_string_of(bar_type);
        assert!(!atomic_is_contained(f, parent, child, &symbols));
    });
}

#[test]
fn class_string_of_unrelated_classes_disjoint() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let foo_named = f.t_named("Foo");
        let foo_type = f.u(foo_named);
        let foo = f.t_class_string_of(foo_type);
        let bar_named = f.t_named("Bar");
        let bar_type = f.u(bar_named);
        let bar = f.t_class_string_of(bar_type);
        assert!(!atomic_is_contained(f, foo, bar, &symbols));
        assert!(!atomic_is_contained(f, bar, foo, &symbols));
    });
}

#[test]
fn class_string_kinds_disjoint() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let named = f.t_named("Foo");
        let named_type = f.u(named);
        let class_of_foo = f.t_class_string_of(named_type);
        let interface_of_foo = f.t_interface_string_of(named_type);
        assert!(!atomic_is_contained(f, class_of_foo, interface_of_foo, &symbols));
        assert!(!atomic_is_contained(f, interface_of_foo, class_of_foo, &symbols));
    });
}

#[test]
fn lit_string_of_valid_class_name_fits_class_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let foo = f.t_lit_string("Foo");
        assert_atomic_subtype(f, foo, class_string);
        let service = f.t_lit_string("App\\Service");
        assert_atomic_subtype(f, service, class_string);
    });
}

#[test]
fn lit_string_of_invalid_class_name_does_not_fit_class_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        for name in ["", "1Foo", "Foo Bar", "Foo\\"] {
            let literal = f.t_lit_string(name);
            assert_atomic_not_subtype(f, literal, class_string);
        }
    });
}

#[test]
fn lit_string_in_class_string_of_matching_class() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let named = f.t_named("Foo");
        let named_type = f.u(named);
        let foo = f.t_class_string_of(named_type);
        let literal = f.t_lit_string("Foo");
        assert!(atomic_is_contained(f, literal, foo, &symbols));
    });
}

#[test]
fn lit_string_of_descendant_in_class_string_of_parent() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo {} class Bar extends Foo {}");
        let named = f.t_named("Foo");
        let named_type = f.u(named);
        let foo = f.t_class_string_of(named_type);
        let literal = f.t_lit_string("Bar");
        assert!(atomic_is_contained(f, literal, foo, &symbols));
    });
}

#[test]
fn lit_string_of_pure_enum_routes_through_enum_kind() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Color {}");
        let any_enum = f.t_enum_string();
        let literal = f.t_lit_string("Color");
        assert!(atomic_is_contained(f, literal, any_enum, &symbols));
    });
}

#[test]
fn lit_string_of_class_does_not_fit_enum_string_container() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo {}");
        let any_enum = f.t_enum_string();
        let literal = f.t_lit_string("Foo");
        assert!(!atomic_is_contained(f, literal, any_enum, &symbols));
    });
}
