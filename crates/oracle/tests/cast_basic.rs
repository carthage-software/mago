mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;
use mago_oracle::ty::cast;
use mago_oracle::ty::cast::CastFlag;
use mago_oracle::ty::cast::CastTarget;
use mago_oracle::ty::well_known;

#[test]
fn cast_int_literal_to_int_is_lossless_and_preserves() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let input = f.ui(42);
        let r = cast::cast(input, CastTarget::Int, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        assert!(!r.flags.contains(CastFlag::MayThrow));
        let expected = f.ui(42);
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_float_to_int_is_lossy() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let lit = f.t_lit_float(3.7);
        let input = f.u(lit);
        let r = cast::cast(input, CastTarget::Int, &cb, &mut f.builder);
        assert!(r.flags.contains(CastFlag::Lossy));
        assert!(!r.flags.contains(CastFlag::MayThrow));
        let expected = f.ui(3);
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_numeric_string_literal_to_int_preserves() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let input = f.us("42");
        let r = cast::cast(input, CastTarget::Int, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        let expected = f.ui(42);
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_non_numeric_string_to_int_is_lossy_zero() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let input = f.us("hello");
        let r = cast::cast(input, CastTarget::Int, &cb, &mut f.builder);
        assert!(r.flags.contains(CastFlag::Lossy));
        let expected = f.ui(0);
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_true_to_int_is_one() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let true_atom = f.t_true();
        let input = f.u(true_atom);
        let r = cast::cast(input, CastTarget::Int, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        let expected = f.ui(1);
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_null_to_int_is_zero() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let null = f.null();
        let input = f.u(null);
        let r = cast::cast(input, CastTarget::Int, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        let expected = f.ui(0);
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_object_to_int_may_throw() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let object = f.t_object_any();
        let input = f.u(object);
        let r = cast::cast(input, CastTarget::Int, &cb, &mut f.builder);
        assert!(r.flags.contains(CastFlag::MayThrow));
    });
}

#[test]
fn cast_int_literal_to_string_preserves() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let input = f.ui(42);
        let r = cast::cast(input, CastTarget::String, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        let expected = f.us("42");
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_true_to_string_is_one_literal() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let true_atom = f.t_true();
        let input = f.u(true_atom);
        let r = cast::cast(input, CastTarget::String, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        let expected = f.us("1");
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_false_to_string_is_empty_literal() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let false_atom = f.t_false();
        let input = f.u(false_atom);
        let r = cast::cast(input, CastTarget::String, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        let expected = f.u(well_known::EMPTY_STRING);
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_array_to_string_may_throw() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let empty_array = f.t_empty_array();
        let input = f.u(empty_array);
        let r = cast::cast(input, CastTarget::String, &cb, &mut f.builder);
        assert!(r.flags.contains(CastFlag::MayThrow));
    });
}

#[test]
fn cast_falsy_literals_to_bool_collapse_to_false() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let zero = f.t_lit_int(0);
        let empty_string = f.t_lit_string("");
        let zero_string = f.t_lit_string("0");
        let null = f.null();
        let false_atom = f.t_false();
        for atom in [zero, empty_string, zero_string, null] {
            let input = f.u(atom);
            let r = cast::cast(input, CastTarget::Bool, &cb, &mut f.builder);
            let expected = f.u(false_atom);
            assert_eq!(r.ty, expected, "expected {atom:?} → false");
        }
    });
}

#[test]
fn cast_truthy_literals_to_bool_collapse_to_true() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let one = f.t_lit_int(1);
        let minus_one = f.t_lit_int(-1);
        let hello = f.t_lit_string("hello");
        let true_atom = f.t_true();
        for atom in [one, minus_one, hello] {
            let input = f.u(atom);
            let r = cast::cast(input, CastTarget::Bool, &cb, &mut f.builder);
            let expected = f.u(true_atom);
            assert_eq!(r.ty, expected, "expected {atom:?} → true");
        }
    });
}

#[test]
fn cast_object_to_bool_is_always_true() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let object = f.t_object_any();
        let input = f.u(object);
        let r = cast::cast(input, CastTarget::Bool, &cb, &mut f.builder);
        let true_atom = f.t_true();
        let expected = f.u(true_atom);
        assert_eq!(r.ty, expected);
        assert!(!r.flags.contains(CastFlag::Lossy));
    });
}

#[test]
fn cast_general_int_to_bool_widens_to_bool() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let int = f.t_int();
        let input = f.u(int);
        let r = cast::cast(input, CastTarget::Bool, &cb, &mut f.builder);
        assert_eq!(r.ty, well_known::TYPE_BOOL);
    });
}

#[test]
fn cast_int_to_float_is_lossless() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let input = f.ui(42);
        let r = cast::cast(input, CastTarget::Float, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        let lit = f.t_lit_float(42.0);
        let expected = f.u(lit);
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_null_to_array_is_empty_array() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let null = f.null();
        let input = f.u(null);
        let r = cast::cast(input, CastTarget::Array, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        let empty_array = f.t_empty_array();
        let expected = f.u(empty_array);
        assert_eq!(r.ty, expected);
    });
}

#[test]
fn cast_int_to_array_is_single_element_list() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let input = f.ui(1);
        let r = cast::cast(input, CastTarget::Array, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy), "(array) 1 == [1] loses nothing");
        assert!(!r.flags.contains(CastFlag::MayThrow));
        assert_eq!(r.ty.atoms.len(), 1);
        let Atom::List(payload) = r.ty.atoms[0] else { panic!("expected a list, got {:?}", r.ty.atoms[0]) };
        let Some(known) = payload.known_elements else { panic!("expected a single known element") };
        assert_eq!(known.len(), 1);
        let value = known[0].value;
        assert_eq!(value, f.ui(1), "the sole element is the literal 1");
    });
}

#[test]
fn cast_string_to_array_is_single_element_list() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let input = f.us("hi");
        let r = cast::cast(input, CastTarget::Array, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::Lossy));
        let Atom::List(payload) = r.ty.atoms[0] else { panic!("expected a list") };
        let Some(known) = payload.known_elements else { panic!("expected a single known element") };
        assert_eq!(known.len(), 1);
        let value = known[0].value;
        assert_eq!(value, f.us("hi"));
    });
}

#[test]
fn cast_object_with_tostring_to_string_is_lossless() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Stringy { public function __toString() {} }");
        let object = f.t_named("Stringy");
        let input = f.u(object);
        let r = cast::cast(input, CastTarget::String, &symbols, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::MayThrow), "a class with __toString casts cleanly");
        assert!(!r.flags.contains(CastFlag::Lossy));
        assert_eq!(r.ty, well_known::TYPE_STRING);
    });
}

#[test]
fn cast_object_without_tostring_to_string_may_throw() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Plain {}");
        let object = f.t_named("Plain");
        let input = f.u(object);
        let r = cast::cast(input, CastTarget::String, &symbols, &mut f.builder);
        assert!(r.flags.contains(CastFlag::MayThrow), "no __toString means the cast may throw");
    });
}

#[test]
fn cast_has_method_tostring_to_string_is_lossless() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let has_to_string = f.t_has_method("__toString");
        let input = f.u(has_to_string);
        let r = cast::cast(input, CastTarget::String, &cb, &mut f.builder);
        assert!(!r.flags.contains(CastFlag::MayThrow), "an object known to have __toString casts cleanly");
        assert_eq!(r.ty, well_known::TYPE_STRING);
    });
}

#[test]
fn cast_object_to_object_is_lossless_passthrough() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let r = cast::cast(foo, CastTarget::Object, &cb, &mut f.builder);
        assert_eq!(r.ty, foo);
        assert!(!r.flags.contains(CastFlag::Lossy));
    });
}

#[test]
fn cast_int_to_object_is_lossy_stdclass() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let input = f.ui(1);
        let r = cast::cast(input, CastTarget::Object, &cb, &mut f.builder);
        assert!(r.flags.contains(CastFlag::Lossy));
        let atoms = r.ty.atoms;
        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms[0].kind(), AtomKind::Object);
    });
}

#[test]
fn cast_distributes_over_union_with_worst_classification() {
    fixture(|f| {
        let cb = empty_symbol_table(f.arena);
        let lit = f.t_lit_int(42);
        let object = f.t_object_any();
        let input = f.u_many(vec![lit, object]);
        let r = cast::cast(input, CastTarget::Int, &cb, &mut f.builder);
        assert!(r.flags.contains(CastFlag::MayThrow), "object branch propagates may-throw flag");
    });
}
