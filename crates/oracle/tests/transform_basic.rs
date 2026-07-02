mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;
use mago_oracle::ty::transform;
use mago_oracle::ty::well_known;

#[test]
fn map_no_op_returns_same_handle() {
    fixture(|f| {
        let ty = well_known::TYPE_INT_OR_STRING;
        let result = transform::map(ty, |atom| atom, &mut f.builder);
        assert_eq!(result, ty);
    });
}

#[test]
fn map_replaces_each_top_level_element() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let float = f.t_float();
        let ty = f.u_many(vec![int, string]);
        let result = transform::map(ty, |atom| if atom == int { float } else { atom }, &mut f.builder);
        let expected = f.u_many(vec![float, string]);
        assert_eq!(result, expected);
    });
}

#[test]
fn map_descends_into_list_element_type() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let inner = f.u(int);
        let list_atom = f.t_list(inner, false);
        let list = f.u(list_atom);
        let result = transform::map(list, |atom| if atom == int { string } else { atom }, &mut f.builder);
        let expected_inner = f.u(string);
        let expected_atom = f.t_list(expected_inner, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn map_descends_into_object_type_args() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let generic_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let generic = f.u(generic_atom);
        let result = transform::map(generic, |atom| if atom == int { string } else { atom }, &mut f.builder);
        let expected_atom = f.t_generic_named("Box", vec![well_known::TYPE_STRING]);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn map_descends_into_iterable_key_and_value() {
    fixture(|f| {
        let int = f.t_int();
        let float = f.t_float();
        let iter_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let iter = f.u(iter_atom);
        let result = transform::map(iter, |atom| if atom == int { float } else { atom }, &mut f.builder);
        let expected_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_FLOAT);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn map_descends_into_keyed_array_value_param() {
    fixture(|f| {
        let int = f.t_int();
        let float = f.t_float();
        let array_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let array = f.u(array_atom);
        let result = transform::map(array, |atom| if atom == int { float } else { atom }, &mut f.builder);
        let expected_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_FLOAT, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn map_post_order_sees_rebuilt_children() {
    fixture(|f| {
        let int = f.t_int();
        let inner = f.u(int);
        let list_atom = f.t_list(inner, false);
        let list = f.u(list_atom);
        let mut seen: Vec<Atom<'_>> = Vec::new();
        let result = transform::map(
            list,
            |atom| {
                seen.push(atom);
                atom
            },
            &mut f.builder,
        );
        assert_eq!(result, list);
        let Some(int_index) = seen.iter().position(|atom| *atom == int) else { panic!("int leaf seen") };
        let Some(list_index) = seen.iter().position(|atom| atom.kind() == AtomKind::List) else { panic!("list seen") };
        assert!(int_index < list_index, "post-order: leaf must be visited before its container");
    });
}

#[test]
fn flat_map_one_to_many_explodes_top_level() {
    fixture(|f| {
        let five = f.t_lit_int(5);
        let low = f.t_int_range(0, 4);
        let high = f.t_int_range(6, 10);
        let ty = f.u(five);
        let result =
            transform::flat_map(ty, |atom| if atom == five { vec![low, high] } else { vec![atom] }, &mut f.builder);
        let expected = f.u_many(vec![low, high]);
        assert_eq!(result, expected);
    });
}

#[test]
fn flat_map_one_to_zero_drops_element() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let ty = f.u_many(vec![int, string]);
        let result =
            transform::flat_map(ty, |atom| if atom == string { Vec::new() } else { vec![atom] }, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn flat_map_inside_list_explodes_nested_element_type() {
    fixture(|f| {
        let five = f.t_lit_int(5);
        let low = f.t_int_range(0, 4);
        let high = f.t_int_range(6, 10);
        let inner = f.u(five);
        let list_atom = f.t_list(inner, false);
        let list = f.u(list_atom);
        let result =
            transform::flat_map(list, |atom| if atom == five { vec![low, high] } else { vec![atom] }, &mut f.builder);
        let expected_inner = f.u_many(vec![low, high]);
        let expected_atom = f.t_list(expected_inner, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn filter_map_drops_when_returning_none() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let null = f.null();
        let ty = f.u_many(vec![int, string, null]);
        let result = transform::filter_map(ty, |atom| if atom == null { None } else { Some(atom) }, &mut f.builder);
        let expected = f.u_many(vec![int, string]);
        assert_eq!(result, expected);
    });
}

#[test]
fn filter_drops_predicate_false() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let null = f.null();
        let ty = f.u_many(vec![int, string, null]);
        let result = transform::filter(ty, |atom| *atom != null, &mut f.builder);
        let expected = f.u_many(vec![int, string]);
        assert_eq!(result, expected);
    });
}

#[test]
fn filter_emptying_a_level_collapses_to_never() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let ty = f.u_many(vec![int, string]);
        let result = transform::filter(ty, |_| false, &mut f.builder);
        assert_eq!(result, well_known::TYPE_NEVER);
    });
}

#[test]
fn filter_emptying_a_nested_level_yields_never_at_that_level() {
    fixture(|f| {
        let int = f.t_int();
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let list = f.u(list_atom);
        let result = transform::filter(list, |atom| *atom != int, &mut f.builder);
        let expected_atom = f.t_list(well_known::TYPE_NEVER, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn map_descends_into_class_like_string_constraint() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let foo_type = f.u(foo);
        let constrained = f.t_class_string_of(foo_type);
        let ty = f.u(constrained);
        let result = transform::map(ty, |atom| if atom == foo { bar } else { atom }, &mut f.builder);
        let bar_type = f.u(bar);
        let expected_inner = f.t_class_string_of(bar_type);
        let expected = f.u(expected_inner);
        assert_eq!(result, expected);
    });
}

#[test]
fn map_descends_into_generic_parameter_constraint() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let param = f.t_template_of("Foo", "T", well_known::TYPE_INT);
        let ty = f.u(param);
        let result = transform::map(ty, |atom| if atom == int { string } else { atom }, &mut f.builder);
        let expected_param = f.t_template_of("Foo", "T", well_known::TYPE_STRING);
        let expected = f.u(expected_param);
        assert_eq!(result, expected);
    });
}
