mod common;

use common::*;

use mago_flags::U8Flags;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeKind;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::atom::payload::scalar::string::StringRefinementFlag;
use mago_oracle::ty::well_known;
use mago_oracle::ty::widen;

fn t_string_with<'arena>(
    f: &mut Fixture<'_, 'arena>,
    literal: StringLiteral<'arena>,
    casing: StringCasing,
    flags: U8Flags<StringRefinementFlag>,
) -> Atom<'arena> {
    f.builder.string(StringAtom { literal, casing, flags })
}

fn t_class_string_lit<'arena>(f: &mut Fixture<'_, 'arena>, name: &str, kind: ClassLikeKind) -> Atom<'arena> {
    let value = f.name(name);

    f.builder.class_like_string(ClassLikeStringAtom { kind, specifier: ClassLikeStringSpecifier::Literal { value } })
}

fn t_class_string_any<'arena>(f: &mut Fixture<'_, 'arena>, kind: ClassLikeKind) -> Atom<'arena> {
    f.builder.class_like_string(ClassLikeStringAtom { kind, specifier: ClassLikeStringSpecifier::Any })
}

fn ty_of<'arena>(f: &mut Fixture<'_, 'arena>, atom: Atom<'arena>) -> Type<'arena> {
    f.u(atom)
}

#[test]
fn scalars_widens_int_literal_to_int() {
    fixture(|f| {
        let input = f.ui(42);
        let result = widen::scalars(input, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn scalars_widens_int_range_to_int() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        let input = f.u(range);
        let result = widen::scalars(input, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn scalars_widens_positive_int_to_int() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let input = f.u(positive);
        let result = widen::scalars(input, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn scalars_widens_float_literal_to_float() {
    fixture(|f| {
        let lit = f.t_lit_float(1.5);
        let input = f.u(lit);
        let result = widen::scalars(input, &mut f.builder);
        assert_eq!(result, well_known::TYPE_FLOAT);
    });
}

#[test]
fn scalars_widens_string_literal_to_string() {
    fixture(|f| {
        let input = f.us("foo");
        let result = widen::scalars(input, &mut f.builder);
        assert_eq!(result, well_known::TYPE_STRING);
    });
}

#[test]
fn scalars_widens_non_empty_string_to_string() {
    fixture(|f| {
        let non_empty = f.t_non_empty_string();
        let input = f.u(non_empty);
        let result = widen::scalars(input, &mut f.builder);
        assert_eq!(result, well_known::TYPE_STRING);
    });
}

#[test]
fn scalars_widens_truthy_string_to_string() {
    fixture(|f| {
        let truthy = f.t_truthy_string();
        let input = f.u(truthy);
        let result = widen::scalars(input, &mut f.builder);
        assert_eq!(result, well_known::TYPE_STRING);
    });
}

#[test]
fn scalars_widens_true_and_false_to_bool() {
    fixture(|f| {
        let true_atom = f.t_true();
        let true_ty = f.u(true_atom);
        assert_eq!(widen::scalars(true_ty, &mut f.builder), well_known::TYPE_BOOL);
        let false_atom = f.t_false();
        let false_ty = f.u(false_atom);
        assert_eq!(widen::scalars(false_ty, &mut f.builder), well_known::TYPE_BOOL);
    });
}

#[test]
fn scalars_widens_class_like_string_to_any_of_same_kind() {
    fixture(|f| {
        let lit = t_class_string_lit(f, "Foo", ClassLikeKind::Class);
        let input = ty_of(f, lit);
        let result = widen::scalars(input, &mut f.builder);
        let any = t_class_string_any(f, ClassLikeKind::Class);
        let expected = ty_of(f, any);
        assert_eq!(result, expected);
    });
}

#[test]
fn scalars_preserves_class_like_string_kind() {
    fixture(|f| {
        let interface_lit = t_class_string_lit(f, "Foo", ClassLikeKind::Interface);
        let input = ty_of(f, interface_lit);
        let result = widen::scalars(input, &mut f.builder);
        let any = t_class_string_any(f, ClassLikeKind::Interface);
        let expected = ty_of(f, any);
        assert_eq!(result, expected);
    });
}

#[test]
fn scalars_leaves_resource_alone() {
    fixture(|f| {
        let open = f.t_open_resource();
        let input = f.u(open);
        let result = widen::scalars(input, &mut f.builder);
        let expected = f.u(open);
        assert_eq!(result, expected);
    });
}

#[test]
fn scalars_descends_into_list_element() {
    fixture(|f| {
        let literal = f.ui(42);
        let list_atom = f.t_list(literal, false);
        let list = f.u(list_atom);
        let result = widen::scalars(list, &mut f.builder);
        let expected_atom = f.t_list(well_known::TYPE_INT, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn scalars_descends_into_object_type_args() {
    fixture(|f| {
        let lit = f.ui(42);
        let generic_atom = f.t_generic_named("Box", vec![lit]);
        let generic = f.u(generic_atom);
        let result = widen::scalars(generic, &mut f.builder);
        let expected_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_widens_int_literal_to_int() {
    fixture(|f| {
        let forty_two = f.ui(42);
        assert_eq!(widen::literals(forty_two, &mut f.builder), well_known::TYPE_INT);
        let zero = f.ui(0);
        assert_eq!(widen::literals(zero, &mut f.builder), well_known::TYPE_INT);
        let minus_one = f.ui(-1);
        assert_eq!(widen::literals(minus_one, &mut f.builder), well_known::TYPE_INT);
    });
}

#[test]
fn literals_widens_unspecified_literal_int_to_int() {
    fixture(|f| {
        let unspecified = f.t_int_unspec_lit();
        let lit_int = f.u(unspecified);
        assert_eq!(widen::literals(lit_int, &mut f.builder), well_known::TYPE_INT);
    });
}

#[test]
fn literals_preserves_int_range() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        let r = f.u(range);
        assert_eq!(widen::literals(r, &mut f.builder), r);
    });
}

#[test]
fn literals_preserves_positive_int_dominator() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let p = f.u(positive);
        assert_eq!(widen::literals(p, &mut f.builder), p);
    });
}

#[test]
fn literals_widens_float_literal_to_float() {
    fixture(|f| {
        let one_and_half = f.t_lit_float(1.5);
        let first = f.u(one_and_half);
        assert_eq!(widen::literals(first, &mut f.builder), well_known::TYPE_FLOAT);
        let zero = f.t_lit_float(0.0);
        let second = f.u(zero);
        assert_eq!(widen::literals(second, &mut f.builder), well_known::TYPE_FLOAT);
    });
}

#[test]
fn literals_widens_unspecified_literal_float_to_float() {
    fixture(|f| {
        let unspecified = f.t_unspec_lit_float();
        let lf = f.u(unspecified);
        assert_eq!(widen::literals(lf, &mut f.builder), well_known::TYPE_FLOAT);
    });
}

#[test]
fn literals_widens_true_to_bool() {
    fixture(|f| {
        let true_atom = f.t_true();
        let true_ty = f.u(true_atom);
        assert_eq!(widen::literals(true_ty, &mut f.builder), well_known::TYPE_BOOL);
        let false_atom = f.t_false();
        let false_ty = f.u(false_atom);
        assert_eq!(widen::literals(false_ty, &mut f.builder), well_known::TYPE_BOOL);
    });
}

#[test]
fn literals_widens_string_to_non_empty_truthy_lowercase() {
    fixture(|f| {
        let input = f.us("foo");
        let result = widen::literals(input, &mut f.builder);
        let expected_atom = t_string_with(
            f,
            StringLiteral::None,
            StringCasing::Lowercase,
            U8Flags::empty().with(StringRefinementFlag::NonEmpty).with(StringRefinementFlag::Truthy),
        );
        let expected = ty_of(f, expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_widens_uppercase_string_correctly() {
    fixture(|f| {
        let input = f.us("FOO");
        let result = widen::literals(input, &mut f.builder);
        let expected_atom = t_string_with(
            f,
            StringLiteral::None,
            StringCasing::Uppercase,
            U8Flags::empty().with(StringRefinementFlag::NonEmpty).with(StringRefinementFlag::Truthy),
        );
        let expected = ty_of(f, expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_widens_mixed_case_string_without_casing() {
    fixture(|f| {
        let input = f.us("FooBar");
        let result = widen::literals(input, &mut f.builder);
        let expected_atom = t_string_with(
            f,
            StringLiteral::None,
            StringCasing::Unspecified,
            U8Flags::empty().with(StringRefinementFlag::NonEmpty).with(StringRefinementFlag::Truthy),
        );
        let expected = ty_of(f, expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_widens_zero_string_to_non_empty_non_truthy() {
    fixture(|f| {
        let input = f.us("0");
        let result = widen::literals(input, &mut f.builder);
        let expected_atom = t_string_with(
            f,
            StringLiteral::None,
            StringCasing::Unspecified,
            U8Flags::empty().with(StringRefinementFlag::NonEmpty).with(StringRefinementFlag::Numeric),
        );
        let expected = ty_of(f, expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_widens_numeric_string() {
    fixture(|f| {
        let input = f.us("42");
        let result = widen::literals(input, &mut f.builder);
        let expected_atom = t_string_with(
            f,
            StringLiteral::None,
            StringCasing::Unspecified,
            U8Flags::empty()
                .with(StringRefinementFlag::NonEmpty)
                .with(StringRefinementFlag::Truthy)
                .with(StringRefinementFlag::Numeric),
        );
        let expected = ty_of(f, expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_widens_negative_numeric_string() {
    fixture(|f| {
        let input = f.us("-1");
        let result = widen::literals(input, &mut f.builder);
        let expected_atom = t_string_with(
            f,
            StringLiteral::None,
            StringCasing::Unspecified,
            U8Flags::empty()
                .with(StringRefinementFlag::NonEmpty)
                .with(StringRefinementFlag::Truthy)
                .with(StringRefinementFlag::Numeric),
        );
        let expected = ty_of(f, expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_widens_decimal_numeric_string() {
    fixture(|f| {
        let input = f.us("1.5");
        let result = widen::literals(input, &mut f.builder);
        let expected_atom = t_string_with(
            f,
            StringLiteral::None,
            StringCasing::Unspecified,
            U8Flags::empty()
                .with(StringRefinementFlag::NonEmpty)
                .with(StringRefinementFlag::Truthy)
                .with(StringRefinementFlag::Numeric),
        );
        let expected = ty_of(f, expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_widens_empty_string_to_string_dominator() {
    fixture(|f| {
        let input = f.us("");
        let result = widen::literals(input, &mut f.builder);
        assert_eq!(result, well_known::TYPE_STRING);
    });
}

#[test]
fn literals_preserves_non_empty_string() {
    fixture(|f| {
        let non_empty = f.t_non_empty_string();
        let nes = f.u(non_empty);
        assert_eq!(widen::literals(nes, &mut f.builder), nes);
    });
}

#[test]
fn literals_preserves_truthy_string() {
    fixture(|f| {
        let truthy = f.t_truthy_string();
        let ts = f.u(truthy);
        assert_eq!(widen::literals(ts, &mut f.builder), ts);
    });
}

#[test]
fn literals_widens_class_like_string_literal_to_any() {
    fixture(|f| {
        let lit = t_class_string_lit(f, "Foo", ClassLikeKind::Class);
        let input = ty_of(f, lit);
        let result = widen::literals(input, &mut f.builder);
        let any = t_class_string_any(f, ClassLikeKind::Class);
        let expected = ty_of(f, any);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_descends_into_list_element() {
    fixture(|f| {
        let literal = f.ui(42);
        let list_atom = f.t_list(literal, false);
        let list = f.u(list_atom);
        let result = widen::literals(list, &mut f.builder);
        let expected_atom = f.t_list(well_known::TYPE_INT, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_descends_into_keyed_array_value() {
    fixture(|f| {
        let lit = f.ui(42);
        let array_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, lit, false);
        let arr = f.u(array_atom);
        let result = widen::literals(arr, &mut f.builder);
        let expected_atom = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_descends_into_iterable_value() {
    fixture(|f| {
        let lit = f.ui(42);
        let iterable_atom = f.t_iterable(well_known::TYPE_STRING, lit);
        let iter = f.u(iterable_atom);
        let result = widen::literals(iter, &mut f.builder);
        let expected_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_descends_into_object_type_args() {
    fixture(|f| {
        let lit = f.ui(42);
        let generic_atom = f.t_generic_named("Box", vec![lit]);
        let generic = f.u(generic_atom);
        let result = widen::literals(generic, &mut f.builder);
        let expected_atom = f.t_generic_named("Box", vec![well_known::TYPE_INT]);
        let expected = f.u(expected_atom);
        assert_eq!(result, expected);
    });
}

#[test]
fn literals_descends_into_generic_parameter_constraint() {
    fixture(|f| {
        let lit = f.ui(42);
        let param = f.t_template_of("Foo", "T", lit);
        let ty = f.u(param);
        let result = widen::literals(ty, &mut f.builder);
        let expected_param = f.t_template_of("Foo", "T", well_known::TYPE_INT);
        let expected = f.u(expected_param);
        assert_eq!(result, expected);
    });
}

#[test]
fn scalars_descends_into_generic_parameter_constraint() {
    fixture(|f| {
        let non_empty = f.t_non_empty_string();
        let non_empty_ty = f.u(non_empty);
        let param = f.t_template_of("Foo", "T", non_empty_ty);
        let ty = f.u(param);
        let result = widen::scalars(ty, &mut f.builder);
        let expected_param = f.t_template_of("Foo", "T", well_known::TYPE_STRING);
        let expected = f.u(expected_param);
        assert_eq!(result, expected);
    });
}

#[test]
fn scalars_no_op_returns_same_handle() {
    fixture(|f| {
        let already_general = well_known::TYPE_INT;
        assert_eq!(widen::scalars(already_general, &mut f.builder), already_general);
    });
}

#[test]
fn literals_no_op_returns_same_handle() {
    fixture(|f| {
        let already_general = well_known::TYPE_INT;
        assert_eq!(widen::literals(already_general, &mut f.builder), already_general);
    });
}

#[test]
fn literals_widens_union_of_literals_to_int() {
    fixture(|f| {
        let one = f.t_lit_int(1);
        let two = f.t_lit_int(2);
        let three = f.t_lit_int(3);
        let union = f.u_many(vec![one, two, three]);
        let result = widen::literals(union, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}

#[test]
fn scalars_widens_union_of_refinements_to_int() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let negative = f.t_negative_int();
        let zero = f.t_lit_int(0);
        let union = f.u_many(vec![positive, negative, zero]);
        let result = widen::scalars(union, &mut f.builder);
        assert_eq!(result, well_known::TYPE_INT);
    });
}
