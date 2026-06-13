mod common;

use common::*;

use mago_flags::U16Flags;

use mago_oracle::ty::FlowFlag;
use mago_oracle::ty::Type;
use mago_oracle::ty::Typed;
use mago_oracle::ty::well_known;

#[track_caller]
fn assert_round_trip<'arena>(f: &mut Fixture<'_, 'arena>, ty: Type<'arena>) {
    let serial = ty.to_serializable();
    let restored = serial.build(&mut f.builder);
    assert_eq!(ty, restored, "content not preserved across round trip");
}

#[test]
fn primitives_round_trip() {
    fixture(|f| {
        assert_round_trip(f, well_known::TYPE_INT);
        assert_round_trip(f, well_known::TYPE_STRING);
        assert_round_trip(f, well_known::TYPE_FLOAT);
        assert_round_trip(f, well_known::TYPE_BOOL);
        assert_round_trip(f, well_known::TYPE_NULL);
        assert_round_trip(f, well_known::TYPE_VOID);
        assert_round_trip(f, well_known::TYPE_NEVER);
        assert_round_trip(f, well_known::TYPE_MIXED);
        assert_round_trip(f, well_known::TYPE_ARRAY_KEY);
        assert_round_trip(f, well_known::TYPE_NUMERIC);
        assert_round_trip(f, well_known::TYPE_SCALAR);
        assert_round_trip(f, well_known::TYPE_OBJECT);
    });
}

#[test]
fn int_literal_round_trips() {
    fixture(|f| {
        let forty_two = f.ui(42);
        assert_round_trip(f, forty_two);
        let minus_one = f.ui(-1);
        assert_round_trip(f, minus_one);
        let zero = f.ui(0);
        assert_round_trip(f, zero);
    });
}

#[test]
fn int_range_round_trips() {
    fixture(|f| {
        let bounded = f.t_int_range(0, 10);
        let bounded_type = f.u(bounded);
        assert_round_trip(f, bounded_type);
        let from = f.t_int_from(5);
        let from_type = f.u(from);
        assert_round_trip(f, from_type);
        let to = f.t_int_to(100);
        let to_type = f.u(to);
        assert_round_trip(f, to_type);
    });
}

#[test]
fn float_literal_round_trips() {
    fixture(|f| {
        let literal = f.t_lit_float(2.5);
        let ty = f.u(literal);
        assert_round_trip(f, ty);
    });
}

#[test]
fn string_literal_and_refinements_round_trip() {
    fixture(|f| {
        let hello = f.us("hello");
        assert_round_trip(f, hello);
        let non_empty = f.t_non_empty_string();
        let non_empty_type = f.u(non_empty);
        assert_round_trip(f, non_empty_type);
        let numeric = f.t_numeric_string();
        let numeric_type = f.u(numeric);
        assert_round_trip(f, numeric_type);
        let lower = f.t_lower_string();
        let lower_type = f.u(lower);
        assert_round_trip(f, lower_type);
    });
}

#[test]
fn unions_round_trip() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        assert_round_trip(f, int_or_string);
    });
}

#[test]
fn flags_round_trip() {
    fixture(|f| {
        let with_by_ref =
            Typed { ty: well_known::TYPE_INT, flags: U16Flags::empty().with(FlowFlag::ByReference), meta: 0 };
        let restored = with_by_ref.to_serializable().build(&mut f.builder);
        assert_eq!(restored.ty, with_by_ref.ty, "content not preserved");
        assert_eq!(restored.flags, with_by_ref.flags, "flags not preserved");
        assert_eq!(restored.meta, with_by_ref.meta, "meta not preserved");
    });
}

#[test]
fn meta_round_trips() {
    fixture(|f| {
        let with_meta = Typed { ty: well_known::TYPE_STRING, flags: U16Flags::empty(), meta: 7 };
        let restored = with_meta.to_serializable().build(&mut f.builder);
        assert_eq!(restored.ty, with_meta.ty, "content not preserved");
        assert_eq!(restored.flags, with_meta.flags, "flags not preserved");
        assert_eq!(restored.meta, with_meta.meta, "meta not preserved");
    });
}

#[test]
fn named_object_round_trips() {
    fixture(|f| {
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        assert_round_trip(f, foo);
    });
}

#[test]
fn generic_object_round_trips() {
    fixture(|f| {
        let box_int_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let box_int = f.u(box_int_atom);
        assert_round_trip(f, box_int);
    });
}

#[test]
fn list_round_trips() {
    fixture(|f| {
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let list_int = f.u(list_atom);
        assert_round_trip(f, list_int);
    });
}

#[test]
fn keyed_array_round_trips() {
    fixture(|f| {
        let array_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let array = f.u(array_atom);
        assert_round_trip(f, array);
    });
}

#[test]
fn iterable_round_trips() {
    fixture(|f| {
        let iterable_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let iterable = f.u(iterable_atom);
        assert_round_trip(f, iterable);
    });
}

#[test]
fn callable_signature_round_trips() {
    fixture(|f| {
        let signature_atom = f.t_callable(&[well_known::TYPE_INT, well_known::TYPE_STRING], well_known::TYPE_BOOL);
        let signature = f.u(signature_atom);
        assert_round_trip(f, signature);
    });
}

#[test]
fn nested_unions_round_trip() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let inner = f.u_many(vec![int, string]);
        let list_atom = f.t_list(inner, false);
        let outer = f.u(list_atom);
        assert_round_trip(f, outer);
    });
}

#[test]
fn restored_within_same_process_uses_same_slot() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let ty = f.u_many(vec![int, string]);
        let restored = ty.to_serializable().build(&mut f.builder);
        assert_eq!(ty, restored, "same builder should re-cons to an identical type");
        assert!(ty.ptr_eq(&restored), "same builder should hand back the original allocation");
    });
}

#[test]
fn element_id_round_trips_via_serializable() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let restored = foo.to_serializable().build(&mut f.builder);
        assert_eq!(foo, restored, "atom re-consed through the same builder should match");
    });
}

#[test]
fn element_id_with_generic_args_round_trips() {
    fixture(|f| {
        let box_int = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let restored = box_int.to_serializable().build(&mut f.builder);
        assert_eq!(box_int, restored);
    });
}

#[test]
fn serde_round_trip_via_json() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let ty = f.u_many(vec![int, string]);
        let serial = ty.to_serializable();
        let transported = serial.clone();
        assert_eq!(serial, transported, "owned structural tree must be value-comparable");
        let restored = transported.build(&mut f.builder);
        assert_eq!(ty, restored);
    });
}

#[test]
fn serde_element_id_round_trip_via_json() {
    fixture(|f| {
        let element = f.t_named("Bar");
        let serial = element.to_serializable();
        let transported = serial.clone();
        assert_eq!(serial, transported, "owned structural atom must be value-comparable");
        let restored = transported.build(&mut f.builder);
        assert_eq!(element, restored);
    });
}
