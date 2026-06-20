mod common;

use common::*;

use mago_oracle::ty::well_known;

#[test]
fn has_method_reflexive() {
    fixture(|f| {
        let has_foo = f.t_has_method("foo");
        assert!(atomic_is_contained(f, has_foo, has_foo, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn distinct_has_methods_dont_refine() {
    fixture(|f| {
        let foo = f.t_has_method("foo");
        let bar = f.t_has_method("bar");
        assert!(!atomic_is_contained(f, foo, bar, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn named_class_with_method_refines_has_method() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo { public function bar() {} }");
        let foo = f.t_named("Foo");
        let has_bar = f.t_has_method("bar");
        assert!(atomic_is_contained(f, foo, has_bar, &symbols));
    });
}

#[test]
fn named_class_without_method_does_not_refine_has_method() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo {}");
        let foo = f.t_named("Foo");
        let has_bar = f.t_has_method("bar");
        assert!(!atomic_is_contained(f, foo, has_bar, &symbols));
    });
}

#[test]
fn inherited_method_satisfies_has_method() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Animal { public function name() {} } class Dog extends Animal {}");
        let dog = f.t_named("Dog");
        let has_name = f.t_has_method("name");
        assert!(atomic_is_contained(f, dog, has_name, &symbols));
    });
}

#[test]
fn unrelated_class_method_does_not_satisfy() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo { public function speak() {} } class Bar {}");
        let bar = f.t_named("Bar");
        let has_speak = f.t_has_method("speak");
        assert!(!atomic_is_contained(f, bar, has_speak, &symbols));
    });
}

#[test]
fn object_any_does_not_refine_has_method() {
    fixture(|f| {
        let object = f.t_object_any();
        let has_foo = f.t_has_method("foo");
        assert!(!atomic_is_contained(f, object, has_foo, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn has_method_refines_object_any() {
    fixture(|f| {
        let has_foo = f.t_has_method("foo");
        let object = f.t_object_any();
        assert!(atomic_is_contained(f, has_foo, object, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn has_method_does_not_refine_named() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo { public function bar() {} }");
        let has_bar = f.t_has_method("bar");
        let foo = f.t_named("Foo");
        assert!(!atomic_is_contained(f, has_bar, foo, &symbols));
    });
}

#[test]
fn has_property_reflexive() {
    fixture(|f| {
        let has_name = f.t_has_property("name");
        assert!(atomic_is_contained(f, has_name, has_name, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn named_class_with_property_refines_has_property() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo { public string $name; }");
        let foo = f.t_named("Foo");
        let has_name = f.t_has_property("name");
        assert!(atomic_is_contained(f, foo, has_name, &symbols));
    });
}

#[test]
fn inherited_property_satisfies_has_property() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Animal { public string $name; } class Dog extends Animal {}");
        let dog = f.t_named("Dog");
        let has_name = f.t_has_property("name");
        assert!(atomic_is_contained(f, dog, has_name, &symbols));
    });
}

#[test]
fn distinct_has_properties_dont_refine() {
    fixture(|f| {
        let has_name = f.t_has_property("name");
        let has_age = f.t_has_property("age");
        assert!(!atomic_is_contained(f, has_name, has_age, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn shape_reflexive() {
    fixture(|f| {
        let shape = f.t_object_shape(&[("name", well_known::TYPE_STRING, false)], false);
        assert!(atomic_is_contained(f, shape, shape, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn shape_with_lit_refines_shape_with_general_via_value_covariance() {
    fixture(|f| {
        let thirty = f.ui(30);
        let literal_shape = f.t_object_shape(&[("age", thirty, false)], false);
        let int_type = f.u(f.t_int());
        let general_shape = f.t_object_shape(&[("age", int_type, false)], false);
        assert!(atomic_is_contained(f, literal_shape, general_shape, &empty_symbol_table(f.arena)));
        assert!(!atomic_is_contained(f, general_shape, literal_shape, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn shape_required_in_optional_container_refines() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let required = f.t_object_shape(&[("name", string_type, false)], false);
        let optional = f.t_object_shape(&[("name", string_type, true)], false);
        assert!(atomic_is_contained(f, required, optional, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn shape_optional_does_not_refine_required() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let optional = f.t_object_shape(&[("name", string_type, true)], false);
        let required = f.t_object_shape(&[("name", string_type, false)], false);
        assert!(!atomic_is_contained(f, optional, required, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn shape_missing_required_property_does_not_refine() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let small = f.t_object_shape(&[("a", int_type, false)], false);
        let big = f.t_object_shape(&[("a", int_type, false), ("b", string_type, false)], false);
        assert!(!atomic_is_contained(f, small, big, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn shape_missing_optional_property_still_refines() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let small = f.t_object_shape(&[("a", int_type, false)], false);
        let big = f.t_object_shape(&[("a", int_type, false), ("b", string_type, true)], false);
        assert!(atomic_is_contained(f, small, big, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn sealed_container_rejects_unsealed_input() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let unsealed = f.t_object_shape(&[("a", int_type, false)], false);
        let sealed = f.t_object_shape(&[("a", int_type, false)], true);
        assert!(!atomic_is_contained(f, unsealed, sealed, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn sealed_container_rejects_input_with_extra_keys() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let extras = f.t_object_shape(&[("a", int_type, false), ("b", string_type, false)], true);
        let sealed_a_only = f.t_object_shape(&[("a", int_type, false)], true);
        assert!(!atomic_is_contained(f, extras, sealed_a_only, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn unsealed_container_accepts_input_with_extra_keys() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let extras = f.t_object_shape(&[("a", int_type, false), ("b", string_type, false)], false);
        let unsealed_a_only = f.t_object_shape(&[("a", int_type, false)], false);
        assert!(atomic_is_contained(f, extras, unsealed_a_only, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn named_class_refines_compatible_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class User { public string $name; public int $age; }");
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let shape = f.t_object_shape(&[("name", string_type, false), ("age", int_type, false)], false);
        let user = f.t_named("User");
        assert!(atomic_is_contained(f, user, shape, &symbols));
    });
}

#[test]
fn named_class_missing_required_property_rejects_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class User { public string $name; }");
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let shape = f.t_object_shape(&[("name", string_type, false), ("age", int_type, false)], false);
        let user = f.t_named("User");
        assert!(!atomic_is_contained(f, user, shape, &symbols));
    });
}

#[test]
fn named_class_missing_optional_property_accepts_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class User { public string $name; }");
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let shape = f.t_object_shape(&[("name", string_type, false), ("age", int_type, true)], false);
        let user = f.t_named("User");
        assert!(atomic_is_contained(f, user, shape, &symbols));
    });
}

#[test]
fn named_class_with_more_specific_property_refines_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Const { /** @var 42 */ public $value; }");
        let int_type = f.u(f.t_int());
        let shape = f.t_object_shape(&[("value", int_type, false)], false);
        let constant = f.t_named("Const");
        assert!(atomic_is_contained(f, constant, shape, &symbols));
    });
}

#[test]
fn named_class_with_wrong_property_type_rejects_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Foo { public string $x; }");
        let int_type = f.u(f.t_int());
        let shape = f.t_object_shape(&[("x", int_type, false)], false);
        let foo = f.t_named("Foo");
        assert!(!atomic_is_contained(f, foo, shape, &symbols));
    });
}

#[test]
fn inherited_property_satisfies_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Base { public int $id; } class Sub extends Base {}");
        let int_type = f.u(f.t_int());
        let shape = f.t_object_shape(&[("id", int_type, false)], false);
        let sub = f.t_named("Sub");
        assert!(atomic_is_contained(f, sub, shape, &symbols));
    });
}
