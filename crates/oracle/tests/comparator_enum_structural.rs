mod common;

use common::*;

#[test]
fn pure_enum_has_name_property() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Status {}");
        let status = f.t_enum("Status");
        let has_name = f.t_has_property("name");
        assert!(atomic_is_contained(f, status, has_name, &symbols));
    });
}

#[test]
fn pure_enum_does_not_have_value_property() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Status {}");
        let status = f.t_enum("Status");
        let has_value = f.t_has_property("value");
        assert!(!atomic_is_contained(f, status, has_value, &symbols));
    });
}

#[test]
fn backed_enum_has_value_property() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Status: string {}");
        let status = f.t_enum("Status");
        let has_value = f.t_has_property("value");
        let has_name = f.t_has_property("name");
        assert!(atomic_is_contained(f, status, has_value, &symbols));
        assert!(atomic_is_contained(f, status, has_name, &symbols));
    });
}

#[test]
fn unknown_enum_rejects_value_but_keeps_name() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let status = f.t_enum("Status");
        let has_value = f.t_has_property("value");
        let has_name = f.t_has_property("name");
        assert!(!atomic_is_contained(f, status, has_value, &symbols));
        assert!(atomic_is_contained(f, status, has_name, &symbols));
    });
}

#[test]
fn backed_string_enum_refines_name_value_string_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Status: string {}");
        let string = f.t_string();
        let string_type = f.u(string);
        let shape = f.t_object_shape(&[("name", string_type, false), ("value", string_type, false)], false);
        let status = f.t_enum("Status");
        assert!(atomic_is_contained(f, status, shape, &symbols));
    });
}

#[test]
fn backed_string_enum_does_not_refine_int_value_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Status: string {}");
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let shape = f.t_object_shape(&[("name", string_type, false), ("value", int_type, false)], false);
        let status = f.t_enum("Status");
        assert!(!atomic_is_contained(f, status, shape, &symbols));
    });
}

#[test]
fn backed_int_enum_refines_int_value_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Priority: int {}");
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let shape = f.t_object_shape(&[("name", string_type, false), ("value", int_type, false)], false);
        let priority = f.t_enum("Priority");
        assert!(atomic_is_contained(f, priority, shape, &symbols));
    });
}

#[test]
fn pure_enum_refines_name_only_shape() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Color {}");
        let string = f.t_string();
        let string_type = f.u(string);
        let shape = f.t_object_shape(&[("name", string_type, false)], false);
        let color = f.t_enum("Color");
        assert!(atomic_is_contained(f, color, shape, &symbols));
    });
}

#[test]
fn pure_enum_does_not_refine_shape_demanding_value() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Color {}");
        let string = f.t_string();
        let string_type = f.u(string);
        let shape = f.t_object_shape(&[("name", string_type, false), ("value", string_type, false)], false);
        let color = f.t_enum("Color");
        assert!(!atomic_is_contained(f, color, shape, &symbols));
    });
}

#[test]
fn pure_enum_refines_shape_with_optional_value() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Color {}");
        let string = f.t_string();
        let string_type = f.u(string);
        let shape = f.t_object_shape(&[("name", string_type, false), ("value", string_type, true)], false);
        let color = f.t_enum("Color");
        assert!(atomic_is_contained(f, color, shape, &symbols));
    });
}

#[test]
fn enum_name_property_is_non_empty_string() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Color {}");
        let non_empty_string = f.t_non_empty_string();
        let non_empty_string_type = f.u(non_empty_string);
        let shape = f.t_object_shape(&[("name", non_empty_string_type, false)], false);
        let color = f.t_enum("Color");
        assert!(atomic_is_contained(f, color, shape, &symbols));
    });
}

#[test]
fn specific_enum_case_refines_shape_with_lit_name() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Color {}");
        let red = f.us("Red");
        let shape = f.t_object_shape(&[("name", red, false)], false);
        let case = f.t_enum_case("Color", "Red");
        assert!(atomic_is_contained(f, case, shape, &symbols));
    });
}

#[test]
fn specific_enum_case_does_not_refine_lit_name_of_different_case() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Color {}");
        let blue = f.us("Blue");
        let shape = f.t_object_shape(&[("name", blue, false)], false);
        let case = f.t_enum_case("Color", "Red");
        assert!(!atomic_is_contained(f, case, shape, &symbols));
    });
}

#[test]
fn enum_does_not_refine_sealed_shape_with_extra_required_key() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Status: string {}");
        let string = f.t_string();
        let int = f.t_int();
        let string_type = f.u(string);
        let int_type = f.u(int);
        let shape = f.t_object_shape(
            &[("name", string_type, false), ("value", string_type, false), ("extra", int_type, false)],
            true,
        );
        let status = f.t_enum("Status");
        assert!(!atomic_is_contained(f, status, shape, &symbols));
    });
}

#[test]
fn enum_refines_sealed_shape_matching_exactly() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php enum Status: string {}");
        let string = f.t_string();
        let string_type = f.u(string);
        let shape = f.t_object_shape(&[("name", string_type, false), ("value", string_type, false)], true);
        let status = f.t_enum("Status");
        assert!(atomic_is_contained(f, status, shape, &symbols));
    });
}

#[test]
fn unknown_enum_rejects_object_shape() {
    fixture(|f| {
        let symbols = empty_symbol_table(f.arena);
        let string = f.t_string();
        let string_type = f.u(string);
        let shape = f.t_object_shape(&[("name", string_type, false)], false);
        let status = f.t_enum("Status");
        assert!(!atomic_is_contained(f, status, shape, &symbols));
    });
}
